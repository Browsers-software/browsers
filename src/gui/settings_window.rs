use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::MessageToMain;
use crate::gui::app::{self, SharedState};
use crate::gui::app_state::{UIBrowser, UIProfileAndIncognito, UISettingsRule};
use crate::gui::generated::{
    BrowserItem, Palette as StdWidgetPalette, RestorableProfile, SettingsRule as SlintSettingsRule,
    SettingsWindow, ThemeChoice,
};
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
    if let Some(win) = state.borrow().settings_window.as_ref().map(|w| w.clone_strong()) {
        win.window().show().ok();
        return;
    }

    let win = SettingsWindow::new().expect("failed to create settings window");
    // needs a real VecModel here, not the generated default - see ui_util::sync_vec_model for why
    win.set_rules(ModelRc::new(VecModel::default()));
    win.set_browsers(ModelRc::new(VecModel::default()));
    win.set_profile_picker_options(ModelRc::new(VecModel::default()));
    win.set_restorable_app_profiles(ModelRc::new(VecModel::default()));
    win.set_directories(ModelRc::new(VecModel::default()));
    state.borrow_mut().settings_window = Some(win.clone_strong());

    app::apply_theme(state);
    wire(state, &win);
    refresh(state);

    state.borrow_mut().extra_windows_open += 1;
    win.window().show().ok();

    {
        let state = state.clone();
        win.window().on_close_requested(move || {
            let new_count = state.borrow().extra_windows_open.saturating_sub(1);
            state.borrow_mut().extra_windows_open = new_count;
            // HideWindow is the only "proceed" variant Slint has - clearing state.settings_window
            // below (the last strong ref) is what actually drops the window
            state.borrow_mut().settings_window = None;
            slint::CloseRequestResponse::HideWindow
        });
    }
}

fn to_lightweight_browser_item(b: &UIBrowser) -> BrowserItem {
    BrowserItem {
        image: Default::default(),
        profile_image: Default::default(),
        browser_name: b.browser_name.clone().into(),
        profile_name: b.profile_name.clone().into(),
        full_name: b.get_full_name().into(),
        show_profile: b.supports_profiles,
        show_incognito: b.supports_incognito,
        hotkey: SharedString::default(),
        show_hotkey: false,
        unique_id: b.unique_id.clone().into(),
        unique_app_id: b.unique_app_id.clone().into(),
        browser_profile_index: b.browser_profile_index as i32,
        filtered_index: b.filtered_index as i32,
        is_first: b.is_first,
        is_last: b.is_last,
        has_priority_ordering: b.has_priority_ordering(),
        supports_profiles: b.supports_profiles,
    }
}

fn resolve_opener_label(opener: &Option<UIProfileAndIncognito>, browsers: &[UIBrowser]) -> String {
    match opener {
        None => "List of Apps".to_string(),
        Some(o) => browsers
            .iter()
            .find(|b| b.unique_id == o.profile)
            .map(|b| b.get_full_name())
            .unwrap_or_else(|| "Unknown".to_string()),
    }
}

// 0 means "List of Apps" (the ComboBox's synthetic first entry, see profile_picker_options
// below); an unmatched profile (uninstalled since the rule was saved) also falls back to 0,
// since there's no model entry to point a ComboBox current-index at otherwise.
fn resolve_opener_index(opener: &Option<UIProfileAndIncognito>, browsers: &[UIBrowser]) -> i32 {
    match opener {
        None => 0,
        Some(o) => browsers
            .iter()
            .position(|b| b.unique_id == o.profile)
            .map(|i| i as i32 + 1)
            .unwrap_or(0),
    }
}

fn resolve_show_incognito(opener: &Option<UIProfileAndIncognito>, browsers: &[UIBrowser]) -> bool {
    match opener {
        None => false,
        Some(o) => browsers
            .iter()
            .find(|b| b.unique_id == o.profile)
            .map(|b| b.supports_incognito)
            .unwrap_or(false),
    }
}

fn to_slint_rule(
    index: i32,
    url_pattern: &str,
    opener: &Option<UIProfileAndIncognito>,
    browsers: &[UIBrowser],
) -> SlintSettingsRule {
    SlintSettingsRule {
        index,
        url_pattern: url_pattern.into(),
        opener_id: opener
            .as_ref()
            .map(|o| o.profile.clone())
            .unwrap_or_default()
            .into(),
        opener_label: resolve_opener_label(opener, browsers).into(),
        opener_index: resolve_opener_index(opener, browsers),
        incognito: opener.as_ref().map(|o| o.incognito).unwrap_or(false),
        show_incognito: resolve_show_incognito(opener, browsers),
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
        .map(|r| to_slint_rule(r.index as i32, &r.url_pattern, &r.opener, &browsers))
        .collect();
    let default_rule = to_slint_rule(-1, "", &settings.default_opener, &browsers);
    let restorable_items: Vec<RestorableProfile> = restorable
        .iter()
        .map(|b| RestorableProfile {
            unique_id: b.unique_id.clone().into(),
            full_name: b.get_full_name().into(),
        })
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
    let directories = crate::gui::about_window::directories_entries();
    crate::gui::ui_util::sync_vec_model(&win.get_directories(), directories);
}

fn save_and_refresh(state: &SharedState, message: MessageToMain) {
    let _ = state.borrow().main_sender.send(message);
    refresh(state);
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
                let next_index = st.ui_settings.rules.len();
                st.ui_settings.rules.push(UISettingsRule {
                    index: next_index,
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
                st.ui_settings.rules.retain(|r| r.index as i32 != idx);
                for (i, r) in st.ui_settings.rules.iter_mut().enumerate() {
                    r.index = i;
                }
            }
            let rules = state.borrow().ui_settings.rules.clone();
            save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
        });
    }
    {
        let state = state.clone();
        win.on_rule_url_changed(move |idx, text| {
            if let Some(r) = state
                .borrow_mut()
                .ui_settings
                .rules
                .iter_mut()
                .find(|r| r.index as i32 == idx)
            {
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
            if idx < 0 {
                let existing_incognito = state
                    .borrow()
                    .ui_settings
                    .default_opener
                    .as_ref()
                    .map(|o| o.incognito)
                    .unwrap_or(false);
                state.borrow_mut().ui_settings.default_opener = opener.map(|mut o| {
                    o.incognito = existing_incognito;
                    o
                });
                let default_opener = state.borrow().ui_settings.default_opener.clone();
                save_and_refresh(&state, MessageToMain::SaveConfigDefaultOpener(default_opener));
            } else {
                if let Some(r) = state
                    .borrow_mut()
                    .ui_settings
                    .rules
                    .iter_mut()
                    .find(|r| r.index as i32 == idx)
                {
                    let existing_incognito =
                        r.opener.as_ref().map(|o| o.incognito).unwrap_or(false);
                    r.opener = opener.map(|mut o| {
                        o.incognito = existing_incognito;
                        o
                    });
                }
                let rules = state.borrow().ui_settings.rules.clone();
                save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
            }
        });
    }
    {
        let state = state.clone();
        win.on_rule_incognito_changed(move |idx, v| {
            if idx < 0 {
                if let Some(o) = state.borrow_mut().ui_settings.default_opener.as_mut() {
                    o.incognito = v;
                }
                let default_opener = state.borrow().ui_settings.default_opener.clone();
                save_and_refresh(&state, MessageToMain::SaveConfigDefaultOpener(default_opener));
            } else {
                if let Some(r) = state
                    .borrow_mut()
                    .ui_settings
                    .rules
                    .iter_mut()
                    .find(|r| r.index as i32 == idx)
                {
                    if let Some(o) = r.opener.as_mut() {
                        o.incognito = v;
                    }
                }
                let rules = state.borrow().ui_settings.rules.clone();
                save_and_refresh(&state, MessageToMain::SaveConfigRules(rules));
            }
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
