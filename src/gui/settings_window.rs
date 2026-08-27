use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::MessageToMain;
use crate::gui::app::{self, SharedState};
use crate::gui::app_state::{UIBrowser, UIProfileAndIncognito, UISettings, UISettingsRule};
use crate::gui::generated::{
    BrowserItem, Palette as StdWidgetPalette, RestorableProfile, SettingsRule as SlintSettingsRule,
    SettingsWindow, ThemeChoice,
};
use crate::gui::main_window;
use crate::gui::theme::ColorScheme;
use crate::utils::ConfiguredTheme;

// forces std-widgets' own Palette (which otherwise just tracks the OS) so an explicit Light/Dark
// choice also applies to native controls like Switch/LineEdit.
// Auto maps to Unknown, un-forcing it.
fn theme_to_color_scheme(theme: ConfiguredTheme) -> ColorScheme {
    match theme {
        ConfiguredTheme::Auto => ColorScheme::Unknown,
        ConfiguredTheme::Light => ColorScheme::Light,
        ConfiguredTheme::Dark => ColorScheme::Dark,
    }
}

pub fn open(state: &SharedState) {
    // focus instead of spawning a second instance if one's already open - state.settings_window
    // gets cleared to None on actual close, so Some here only ever means "currently showing"
    if let Some(win) = state
        .borrow()
        .settings_window
        .as_ref()
        .map(|w| w.clone_strong())
    {
        win.window().show().ok();
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

    // otherwise this renders behind the always-on-top main popup - see app::make_window_floating
    app::make_window_floating(&win);

    {
        let state = state.clone();
        win.window().on_close_requested(move || {
            // HideWindow is the only "proceed" variant Slint has - clearing state.settings_window
            // below (the last strong ref) is what actually drops the window
            state.borrow_mut().settings_window = None;
            slint::CloseRequestResponse::HideWindow
        });
    }
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
    win.global::<StdWidgetPalette>()
        .set_color_scheme(theme_to_color_scheme(settings.visual_settings.theme));
    crate::gui::ui_util::sync_vec_model(&win.get_rules(), rule_items);
    win.set_default_rule(default_rule);
    crate::gui::ui_util::sync_vec_model(&win.get_browsers(), browser_items);
    crate::gui::ui_util::sync_vec_model(
        &win.get_profile_picker_options(),
        profile_picker_options(&browsers),
    );
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
    {
        let state = state.clone();
        win.on_restore_profile(move |id| {
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::RestoreAppProfile(id.to_string()));
        });
    }
    {
        let state = state.clone();
        win.on_theme_changed(move |choice| {
            state.borrow_mut().ui_settings.visual_settings.theme = choice_to_theme(choice);
            let settings = state.borrow().ui_settings.visual_settings.clone();
            app::apply_theme(&state);
            app::refresh_main_window_model(&state);
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
        win.on_set_default_browser(move || {
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::SetBrowsersAsDefaultBrowser);
        });
    }
    {
        let state = state.clone();
        win.on_refresh(move || {
            let _ = state.borrow().main_sender.send(MessageToMain::Refresh);
        });
    }
}
