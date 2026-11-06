use std::str::FromStr;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use url::Url;

use crate::MessageToMain;
use crate::gui::app::{self, SharedState};
use crate::gui::app_state::{UIBrowser, UIProfileAndIncognito, UISettings, UISettingsRule};
use crate::gui::generated::{
    BrowserItem, Palette as StdWidgetPalette, RestorableProfile, SettingsRule as SlintSettingsRule,
    SettingsWindow, ThemeChoice,
};
use crate::gui::main_window;
use crate::gui::theme::configured_color_scheme;
use crate::url_rule;
use crate::utils::{BehavioralConfig, ConfiguredTheme};

pub fn open(state: &SharedState) {
    // state.settings_window gets cleared to None on actual close, so Some here only ever means
    // "currently showing"
    let already_open = state
        .borrow()
        .settings_window
        .as_ref()
        .map(|w| w.clone_strong());
    if app::focus_if_open(already_open) {
        return;
    }

    let win = SettingsWindow::new().expect("failed to create settings window");
    // needs a real VecModel here, not the generated default - see ui_util::sync_vec_model for why
    win.set_rules(ModelRc::new(VecModel::default()));
    win.set_browsers(ModelRc::new(VecModel::default()));
    win.set_profile_picker_options(ModelRc::new(VecModel::default()));
    win.set_restorable_app_profiles(ModelRc::new(VecModel::default()));
    // directories (Config/Cache/Logs/Resources paths) don't change during a run, so this is set
    // once here rather than recomputed on every refresh() call
    win.set_directories(ModelRc::new(VecModel::from(
        crate::gui::about_window::directories_entries(),
    )));
    state.borrow_mut().settings_window = Some(win.clone_strong());

    app::apply_theme(state);
    wire(state, &win);
    refresh(state);

    win.window().show().ok();

    // the main popup can show over full-screen apps, so these need to as well
    app::allow_over_fullscreen_apps(&win);

    app::clear_on_close(&win, state, |st| st.settings_window = None);
}

fn to_lightweight_browser_item(b: &UIBrowser) -> BrowserItem {
    main_window::browser_item(b, Default::default(), Default::default(), String::new(), false)
}

// looks up the browser an opener points at once, instead of three separate scans over
// `browsers` for the label/index/incognito support that all derive from the same match
fn resolve_opener<'a>(
    opener: &Option<UIProfileAndIncognito>,
    browsers: &'a [UIBrowser],
) -> Option<(usize, &'a UIBrowser)> {
    let o = opener.as_ref()?;
    browsers
        .iter()
        .enumerate()
        .find(|(_, b)| b.unique_id == o.profile)
}

fn to_slint_rule(
    index: i32,
    url_pattern: &str,
    opener: &Option<UIProfileAndIncognito>,
    browsers: &[UIBrowser],
) -> SlintSettingsRule {
    let resolved = resolve_opener(opener, browsers);

    let opener_label = match (opener, resolved) {
        (None, _) => "List of Apps".to_string(),
        (Some(_), Some((_, b))) => b.get_full_name(),
        (Some(_), None) => "Unknown".to_string(),
    };
    // 0 means "List of Apps" (the ComboBox's synthetic first entry, see profile_picker_options
    // below); an unmatched profile (uninstalled since the rule was saved) also falls back to 0,
    // since there's no model entry to point a ComboBox current-index at otherwise.
    let opener_index = resolved.map_or(0, |(i, _)| i as i32 + 1);
    let show_incognito = resolved.is_some_and(|(_, b)| b.supports_incognito);

    SlintSettingsRule {
        index,
        url_pattern: url_pattern.into(),
        opener_id: opener
            .as_ref()
            .map(|o| o.profile.clone())
            .unwrap_or_default()
            .into(),
        opener_label: opener_label.into(),
        opener_index,
        incognito: opener.as_ref().map(|o| o.incognito).unwrap_or(false),
        show_incognito,
    }
}

// the ComboBox's model: a synthetic "List of Apps" entry followed by one entry per browser, in
// the same order as the `browsers` property - ProfilePicker relies on that shared ordering to
// map a selected index back to a browser's unique-id without needing to match on its label text.
fn profile_picker_options(browsers: &[UIBrowser]) -> Vec<SharedString> {
    std::iter::once("List of Apps".into())
        .chain(browsers.iter().map(|b| b.get_full_name().into()))
        .collect()
}

fn theme_to_choice(theme: ConfiguredTheme) -> ThemeChoice {
    match theme {
        ConfiguredTheme::Auto => ThemeChoice::Auto,
        ConfiguredTheme::Light => ThemeChoice::Light,
        ConfiguredTheme::Dark => ThemeChoice::Dark,
    }
}

fn choice_to_theme(choice: ThemeChoice) -> ConfiguredTheme {
    match choice {
        ThemeChoice::Auto => ConfiguredTheme::Auto,
        ThemeChoice::Light => ConfiguredTheme::Light,
        ThemeChoice::Dark => ConfiguredTheme::Dark,
    }
}

// "Test a URL": what would happen to `input`, and which rule (index into the rules list, -1 for
// none) decides it. Mirrors route_url/get_rule_for_source_app_and_url in lib.rs: the URL is
// unwrapped first, rules are tried in order and the first match wins, then the default opener.
// A pasted URL has no source app, so rules that need one are skipped.
fn test_url(settings: &UISettings, browsers: &[UIBrowser], input: &str) -> (String, i32) {
    let input = input.trim();
    if input.is_empty() {
        return (String::new(), -1);
    }

    let with_scheme = if input.contains("://") {
        input.to_string()
    } else {
        format!("https://{input}")
    };
    let behavioral = BehavioralConfig {
        unwrap_urls: settings.behavioral_settings.unwrap_urls,
    };
    let cleaned = crate::unwrap_url(&with_scheme, &behavioral);
    let Ok(url) = Url::from_str(&cleaned) else {
        return ("Not a valid URL".to_string(), -1);
    };

    let describe = |opener: &Option<UIProfileAndIncognito>| match resolve_opener(opener, browsers) {
        Some((_, b)) => {
            let incognito = opener.as_ref().is_some_and(|o| o.incognito) && b.supports_incognito;
            let suffix = if incognito { " (Incognito)" } else { "" };
            format!("opens in {}{suffix}", b.get_full_name())
        }
        None => "shows the list of apps".to_string(),
    };

    let mut invalid_rules: Vec<usize> = Vec::new();
    for (i, rule) in settings.rules.iter().enumerate() {
        if rule.get_source_app().is_some() {
            continue;
        }
        let matches = match rule.get_url_pattern() {
            None => true,
            Some(pattern) => match url_rule::try_pattern_matches_url(&pattern, &url) {
                Some(matches) => matches,
                None => {
                    invalid_rules.push(i + 1);
                    false
                }
            },
        };
        if matches {
            let any_url = if rule.get_url_pattern().is_none() {
                " (it has no URL, so it matches every link)"
            } else {
                ""
            };
            let mut text = format!("Rule {} matches{any_url}: {}", i + 1, describe(&rule.opener));
            if cleaned != with_scheme {
                text.push_str(&format!("\nUnwrapped to {cleaned}"));
            }
            return (text, i as i32);
        }
    }

    let mut text = format!("No rule matches: {}", describe(&settings.default_opener));
    if cleaned != with_scheme {
        text.push_str(&format!("\nUnwrapped to {cleaned}"));
    }
    if !invalid_rules.is_empty() {
        let numbers: Vec<String> = invalid_rules.iter().map(|n| n.to_string()).collect();
        text.push_str(&format!("\nSkipped rule(s) with an invalid URL: {}", numbers.join(", ")));
    }
    (text, -1)
}

fn update_test_url_result(state: &SharedState, win: &SettingsWindow) {
    let (text, index) = {
        let st = state.borrow();
        test_url(&st.ui_settings, &st.browsers, win.get_test_url_text().as_str())
    };
    win.set_test_url_result(text.into());
    win.set_test_url_match_index(index);
}

pub fn refresh(state: &SharedState) {
    let st = state.borrow();
    let win = match &st.settings_window {
        Some(w) => w.clone_strong(),
        None => return,
    };
    let settings = st.ui_settings.clone();
    let browsers = st.browsers.clone();
    let restorable = st.restorable_app_profiles.clone();
    drop(st);

    let browser_items: Vec<BrowserItem> =
        browsers.iter().map(to_lightweight_browser_item).collect();
    let rule_items: Vec<SlintSettingsRule> = settings
        .rules
        .iter()
        .enumerate()
        .map(|(i, r)| to_slint_rule(i as i32, &r.url_pattern, &r.opener, &browsers))
        .collect();
    let default_rule = to_slint_rule(-1, "", &settings.default_opener, &browsers);
    let restorable_items: Vec<RestorableProfile> = restorable
        .iter()
        .map(main_window::to_restorable_profile)
        .collect();

    win.set_show_hotkeys(settings.visual_settings.show_hotkeys);
    win.set_quit_on_lost_focus(settings.visual_settings.quit_on_lost_focus);
    win.set_show_quit_on_lost_focus(cfg!(target_os = "macos"));
    win.set_unwrap_urls(settings.behavioral_settings.unwrap_urls);
    crate::gui::ui_util::sync_vec_model(&win.get_restorable_app_profiles(), restorable_items);
    win.set_theme(theme_to_choice(settings.visual_settings.theme));
    win.set_text_scale(settings.visual_settings.text_scale);
    // forces std-widgets' own Palette (which otherwise just tracks the OS) so an explicit
    // Light/Dark choice also applies to native controls like Switch/LineEdit
    win.global::<StdWidgetPalette>()
        .set_color_scheme(configured_color_scheme(settings.visual_settings.theme));
    crate::gui::ui_util::sync_vec_model(&win.get_rules(), rule_items);
    win.set_default_rule(default_rule);
    crate::gui::ui_util::sync_vec_model(&win.get_browsers(), browser_items);
    crate::gui::ui_util::sync_vec_model(
        &win.get_profile_picker_options(),
        profile_picker_options(&browsers),
    );
    // rules or the default opener may have changed under the tested URL
    update_test_url_result(state, &win);
}

fn save_and_refresh(state: &SharedState, message: MessageToMain) {
    let _ = state.borrow().main_sender.send(message);
    refresh(state);
}

// the opener slot idx refers to: the default opener for idx < 0, or the matching rule's opener
// otherwise - shared by on_rule_profile_changed/on_rule_incognito_changed so they only differ in
// what they write into the slot
fn opener_slot_mut(
    settings: &mut UISettings,
    idx: i32,
) -> Option<&mut Option<UIProfileAndIncognito>> {
    if idx < 0 {
        Some(&mut settings.default_opener)
    } else {
        settings.rules.get_mut(idx as usize).map(|r| &mut r.opener)
    }
}

// saves whichever opener_slot_mut(idx) just wrote to - the default opener or the rules list,
// matching the same idx < 0 split
fn save_opener_change(state: &SharedState, idx: i32) {
    if idx < 0 {
        let default_opener = state.borrow().ui_settings.default_opener.clone();
        save_and_refresh(state, MessageToMain::SaveConfigDefaultOpener(default_opener));
    } else {
        let rules = state.borrow().ui_settings.rules.clone();
        save_and_refresh(state, MessageToMain::SaveConfigRules(rules));
    }
}

fn wire(state: &SharedState, win: &SettingsWindow) {
    {
        let state = state.clone();
        win.on_show_hotkeys_changed(move |v| {
            state.borrow_mut().ui_settings.visual_settings.show_hotkeys = v;
            let settings = state.borrow().ui_settings.visual_settings.clone();
            app::refresh_main_window_model(&state);
            save_and_refresh(&state, MessageToMain::SaveConfigUISettings(settings));
        });
    }
    {
        let state = state.clone();
        win.on_quit_on_lost_focus_changed(move |v| {
            state
                .borrow_mut()
                .ui_settings
                .visual_settings
                .quit_on_lost_focus = v;
            let settings = state.borrow().ui_settings.visual_settings.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigUISettings(settings));
        });
    }
    {
        let state = state.clone();
        win.on_unwrap_urls_changed(move |v| {
            state
                .borrow_mut()
                .ui_settings
                .behavioral_settings
                .unwrap_urls = v;
            let settings = state.borrow().ui_settings.behavioral_settings.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigUIBehavioralSettings(settings));
        });
    }
    win.on_restore_profile(app::forward_id_handler(
        state,
        MessageToMain::RestoreAppProfile,
    ));
    {
        let state = state.clone();
        win.on_text_scale_changed(move |scale| {
            state.borrow_mut().ui_settings.visual_settings.text_scale = scale;
            let settings = state.borrow().ui_settings.visual_settings.clone();
            app::apply_text_scale(&state);
            save_and_refresh(&state, MessageToMain::SaveConfigUISettings(settings));
        });
    }
    {
        let state = state.clone();
        win.on_theme_changed(move |choice| {
            state.borrow_mut().ui_settings.visual_settings.theme = choice_to_theme(choice);
            let settings = state.borrow().ui_settings.visual_settings.clone();
            app::apply_theme(&state);
            save_and_refresh(&state, MessageToMain::SaveConfigUISettings(settings));
        });
    }
    {
        let state = state.clone();
        win.on_add_rule(move || {
            {
                let mut st = state.borrow_mut();
                st.ui_settings.rules.push(UISettingsRule {
                    source_app: String::new(),
                    url_pattern: String::new(),
                    opener: None,
                });
            }
            let rules = state.borrow().ui_settings.rules.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
        });
    }
    {
        let state = state.clone();
        win.on_remove_rule(move |idx| {
            {
                let mut st = state.borrow_mut();
                if idx >= 0 && (idx as usize) < st.ui_settings.rules.len() {
                    st.ui_settings.rules.remove(idx as usize);
                }
            }
            let rules = state.borrow().ui_settings.rules.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
        });
    }
    {
        let state = state.clone();
        win.on_rule_url_changed(move |idx, text| {
            if let Some(r) = state.borrow_mut().ui_settings.rules.get_mut(idx as usize) {
                r.url_pattern = text.to_string();
            }
            let rules = state.borrow().ui_settings.rules.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
        });
    }
    {
        let state = state.clone();
        win.on_rule_profile_changed(move |idx, id| {
            let id = id.to_string();
            let opener = if id.is_empty() {
                None
            } else {
                Some(UIProfileAndIncognito {
                    profile: id,
                    incognito: false,
                })
            };

            {
                let mut st = state.borrow_mut();
                if let Some(slot) = opener_slot_mut(&mut st.ui_settings, idx) {
                    let existing_incognito = slot.as_ref().map(|o| o.incognito).unwrap_or(false);
                    *slot = opener.map(|mut o| {
                        o.incognito = existing_incognito;
                        o
                    });
                }
            }
            save_opener_change(&state, idx);
        });
    }
    {
        let state = state.clone();
        win.on_rule_incognito_changed(move |idx, v| {
            {
                let mut st = state.borrow_mut();
                if let Some(o) =
                    opener_slot_mut(&mut st.ui_settings, idx).and_then(|slot| slot.as_mut())
                {
                    o.incognito = v;
                }
            }
            save_opener_change(&state, idx);
        });
    }
    {
        let state = state.clone();
        let win_weak = win.as_weak();
        win.on_test_url_edited(move || {
            if let Some(win) = win_weak.upgrade() {
                update_test_url_result(&state, &win);
            }
        });
    }
    win.on_set_default_browser(app::forward_handler(state, || {
        MessageToMain::SetBrowsersAsDefaultBrowser
    }));
    win.on_refresh(app::forward_handler(state, || MessageToMain::Refresh));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::app_state::UIVisualSettings;

    fn browser(unique_id: &str, name: &str, supports_incognito: bool) -> UIBrowser {
        UIBrowser {
            browser_profile_index: 0,
            is_first: false,
            is_last: false,
            restricted_url_matchers: Default::default(),
            browser_name: name.to_string(),
            profile_name: String::new(),
            supports_profiles: false,
            supports_incognito,
            icon_path: String::new(),
            profile_icon_path: String::new(),
            unique_id: unique_id.to_string(),
            unique_app_id: unique_id.to_string(),
            filtered_index: 0,
        }
    }

    fn opener(profile: &str, incognito: bool) -> Option<UIProfileAndIncognito> {
        Some(UIProfileAndIncognito {
            profile: profile.to_string(),
            incognito,
        })
    }

    fn rule(
        source_app: &str,
        url_pattern: &str,
        opener: Option<UIProfileAndIncognito>,
    ) -> UISettingsRule {
        UISettingsRule {
            source_app: source_app.to_string(),
            url_pattern: url_pattern.to_string(),
            opener,
        }
    }

    fn settings(
        rules: Vec<UISettingsRule>,
        default_opener: Option<UIProfileAndIncognito>,
        unwrap_urls: bool,
    ) -> UISettings {
        UISettings {
            default_opener,
            rules,
            visual_settings: UIVisualSettings {
                show_hotkeys: false,
                quit_on_lost_focus: false,
                theme: ConfiguredTheme::Auto,
                text_scale: 1.0,
            },
            behavioral_settings: crate::gui::app_state::UIBehavioralSettings { unwrap_urls },
        }
    }

    fn browsers() -> Vec<UIBrowser> {
        vec![browser("chrome", "Chrome", true), browser("safari", "Safari", false)]
    }

    #[test]
    fn empty_input_shows_nothing() {
        let s = settings(vec![], None, false);

        assert_eq!(test_url(&s, &browsers(), "  "), (String::new(), -1));
    }

    #[test]
    fn first_matching_rule_wins() {
        let s = settings(
            vec![
                rule("", "example.org", opener("safari", false)),
                rule("", "github.com/**", opener("chrome", true)),
                rule("", "github.com/Browsers-software/**", opener("safari", false)),
            ],
            None,
            false,
        );

        let (text, index) = test_url(&s, &browsers(), "https://github.com/Browsers-software/x");

        assert_eq!(index, 1);
        assert_eq!(text, "Rule 2 matches: opens in Chrome (Incognito)");
    }

    #[test]
    fn scheme_is_optional() {
        let s = settings(vec![rule("", "github.com/**", opener("chrome", false))], None, false);

        let (text, index) = test_url(&s, &browsers(), "github.com/Browsers-software");

        assert_eq!(index, 0);
        assert_eq!(text, "Rule 1 matches: opens in Chrome");
    }

    #[test]
    fn no_match_uses_default_opener() {
        let s = settings(
            vec![rule("", "example.org", opener("safari", false))],
            opener("chrome", false),
            false,
        );

        let (text, index) = test_url(&s, &browsers(), "https://github.com/");

        assert_eq!(index, -1);
        assert_eq!(text, "No rule matches: opens in Chrome");
    }

    #[test]
    fn no_match_and_no_default_shows_list() {
        let s = settings(vec![], None, false);

        let (text, index) = test_url(&s, &browsers(), "https://github.com/");

        assert_eq!(index, -1);
        assert_eq!(text, "No rule matches: shows the list of apps");
    }

    #[test]
    fn rule_without_opener_or_with_missing_browser_shows_list() {
        let s = settings(
            vec![
                rule("", "example.org", None),
                rule("", "github.com", opener("gone", false)),
            ],
            None,
            false,
        );

        assert_eq!(
            test_url(&s, &browsers(), "https://example.org/").0,
            "Rule 1 matches: shows the list of apps"
        );
        assert_eq!(
            test_url(&s, &browsers(), "https://github.com/").0,
            "Rule 2 matches: shows the list of apps"
        );
    }

    #[test]
    fn rule_without_url_matches_every_link() {
        let s = settings(vec![rule("", "", None)], None, false);

        let (text, index) = test_url(&s, &browsers(), "https://github.com/");

        assert_eq!(index, 0);
        assert!(text.starts_with("Rule 1 matches (it has no URL, so it matches every link)"));
    }

    #[test]
    fn rule_needing_a_source_app_is_skipped() {
        let s = settings(
            vec![rule("com.example.Mail", "github.com", opener("safari", false))],
            None,
            false,
        );

        assert_eq!(test_url(&s, &browsers(), "https://github.com/").1, -1);
    }

    #[test]
    fn invalid_pattern_is_skipped_and_reported() {
        let s = settings(
            vec![
                rule("", "github.com/[", opener("safari", false)),
                rule("", "github.com/**", opener("chrome", false)),
            ],
            None,
            false,
        );

        let (text, index) = test_url(&s, &browsers(), "https://github.com/x");

        // the broken rule can't decide, the next one does
        assert_eq!(index, 1);
        assert_eq!(text, "Rule 2 matches: opens in Chrome");

        // and when nothing matches, the broken rule is named
        let (text, index) = test_url(&s, &browsers(), "https://example.org/");
        assert_eq!(index, -1);
        assert!(text.contains("Skipped rule(s) with an invalid URL: 1"));
    }

    #[test]
    fn unwrapped_url_is_what_gets_matched() {
        let wrapped =
            "https://eur01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.org%2Fa";
        let s = settings(vec![rule("", "example.org/**", opener("chrome", false))], None, true);

        let (text, index) = test_url(&s, &browsers(), wrapped);

        assert_eq!(index, 0);
        assert!(text.contains("Unwrapped to https://example.org/a"));
    }

    #[test]
    fn nonsense_is_not_a_valid_url() {
        let s = settings(vec![], None, false);

        assert_eq!(test_url(&s, &browsers(), "http://").0, "Not a valid URL");
    }
}
