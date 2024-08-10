use std::ops::Not;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use druid::commands::{CONFIGURE_WINDOW_SIZE_AND_POSITION, QUIT_APP, SHOW_WINDOW};
use druid::{
    AppDelegate, AppLauncher, Command, Data, DelegateCtx, Env, Event, Handled, KbKey, KeyEvent,
    Lens, Point, Selector, Target, WindowId,
};
use druid::{Application, Code, Modifiers, Monitor, WindowHandle};
use tracing::{debug, info, instrument};
use url::Url;

use crate::gui::main_window::{
    calculate_window_position, recalculate_window_size, COPY_LINK_TO_CLIPBOARD, HIDE_ALL_PROFILES,
    HIDE_PROFILE, MOVE_PROFILE, OPEN_LINK_IN_BROWSER, REFRESH, RESTORE_HIDDEN_PROFILE,
    SET_BROWSERS_AS_DEFAULT_BROWSER, SET_FOCUSED_INDEX, SHOW_ABOUT_DIALOG, SHOW_SETTINGS_DIALOG,
};
use crate::gui::ui::SettingsTab::GENERAL;
use crate::gui::{about_dialog, main_window, settings_window, ui_theme};
use crate::url_rule::UrlGlobMatcher;
use crate::utils::{BehavioralConfig, Config, ConfiguredTheme, ProfileAndOptions, UIConfig};
use crate::{CommonBrowserProfile, MessageToMain};

pub struct UI {
    localizations_basedir: PathBuf,
    main_sender: Sender<MessageToMain>,
    url: String,
    ui_browsers: Arc<Vec<UIBrowser>>,
    filtered_browsers: Arc<Vec<UIBrowser>>,
    restorable_app_profiles: Arc<Vec<UIBrowser>>,
    show_set_as_default: bool,
    ui_settings: UISettings,
}

impl UI {
    pub fn config_to_ui_settings(config: &Config) -> UISettings {
        let ui_settings_rules = config
            .get_rules()
            .iter()
            .enumerate()
            .map(|(i, rule)| UISettingsRule {
                index: i,
                saved: true,
                deleted: false,
                source_app: rule
                    .source_app
                    .as_ref()
                    .map_or("".to_string(), |s| s.clone()),
                url_pattern: rule
                    .url_pattern
                    .as_ref()
                    .map_or("".to_string(), |s| s.clone()),
                opener: Self::map_as_ui_profile(&rule.get_opener()),
            })
            .collect();

        let default_opener = Self::map_as_ui_profile(config.get_default_profile());

        return UISettings {
            tab: GENERAL,
            default_opener: default_opener,
            rules: Arc::new(ui_settings_rules),
            visual_settings: Self::map_as_visual_settings(config.get_ui_config()),
            behavioral_settings: Self::map_as_ui_behavioural_settings(config.get_behavior()),
        };
    }
    fn map_as_visual_settings(ui_config: &UIConfig) -> UIVisualSettings {
        UIVisualSettings {
            show_hotkeys: ui_config.show_hotkeys,
            quit_on_lost_focus: ui_config.quit_on_lost_focus,
            theme: ui_config.theme,
        }
    }

    fn map_as_ui_behavioural_settings(behavior: &BehavioralConfig) -> UIBehavioralSettings {
        UIBehavioralSettings {
            unwrap_urls: behavior.unwrap_urls,
        }
    }

    fn map_as_ui_profile(
        profile_and_options: &Option<ProfileAndOptions>,
    ) -> Option<UIProfileAndIncognito> {
        return profile_and_options.as_ref().map(|p| UIProfileAndIncognito {
            profile: p.profile.clone(),
            incognito: p.incognito,
        });
    }

    pub fn real_to_ui_browsers(all_browser_profiles: &[CommonBrowserProfile]) -> Vec<UIBrowser> {
        if all_browser_profiles.is_empty() {
            return vec![];
        }

        // TODO: this is a bit ugly; we keep profiles with has_priority_ordering() always on top
        //       and everything else comes after; it might make sense to keep them in two separate
        //       vectors (or slices)
        let first_orderable_item_index_maybe = all_browser_profiles
            .iter()
            .position(|b| !b.has_priority_ordering());
        let first_orderable_item_index = first_orderable_item_index_maybe.unwrap_or(0);

        let profiles_count = all_browser_profiles.len();

        return all_browser_profiles
            .iter()
            .enumerate()
            .map(|(i, p)| UIBrowser {
                browser_profile_index: i,
                is_first: i == first_orderable_item_index,
                is_last: i == profiles_count - 1,
                restricted_url_matchers: Arc::new(p.get_restricted_url_matchers().clone()),
                browser_name: p.get_browser_name().to_string(),
                profile_name: p.get_profile_name().to_string(),
                supports_profiles: p.get_browser_common().has_real_profiles(),
                profile_name_maybe: p
                    .get_browser_common()
                    .has_real_profiles()
                    .then(|| p.get_profile_name().to_string()),
                supports_incognito: p.get_browser_common().supports_incognito(),
                icon_path: p.get_browser_icon_path().to_string(),
                profile_icon_path: p
                    .get_profile_icon_path()
                    .map_or("".to_string(), |a| a.to_string()),
                unique_id: p.get_unique_id(),
                unique_app_id: p.get_unique_app_id(),
                filtered_index: i, // TODO: filter against current url
            })
            .collect();
    }

    pub fn new(
        localizations_basedir: PathBuf,
        main_sender: Sender<MessageToMain>,
        url: &str,
        ui_browsers: Vec<UIBrowser>,
        restorable_app_profiles: Vec<UIBrowser>,
        show_set_as_default: bool,
        ui_settings: UISettings,
    ) -> Self {
        let ui_browsers = Arc::new(ui_browsers);
        let filtered_browsers = get_filtered_browsers(&url, &ui_browsers);

        Self {
            localizations_basedir: localizations_basedir,
            main_sender: main_sender.clone(),
            url: url.to_string(),
            ui_browsers: ui_browsers,
            filtered_browsers: Arc::new(filtered_browsers),
            restorable_app_profiles: Arc::new(restorable_app_profiles),
            show_set_as_default: show_set_as_default,
            ui_settings: ui_settings,
        }
    }

    #[instrument(skip_all)]
    pub fn create_app_launcher(&self) -> AppLauncher<UIState> {
        let basedir = self.localizations_basedir.to_str().unwrap().to_string();
        let (mouse_position, monitor) = druid::Screen::get_mouse_position();

        let browser_count = (&self.filtered_browsers).len();
        let main_window1 = main_window::MainWindow::new();
        let main_window = main_window1.create_main_window(browser_count, &mouse_position, &monitor);

        let main_window_id = main_window.id.clone();
        return AppLauncher::with_window(main_window)
            .delegate(UIDelegate {
                main_sender: self.main_sender.clone(),
                windows: vec![main_window_id],
                main_window_id: main_window_id,
                mouse_position: mouse_position.clone(),
                monitor: monitor.clone(),
            })
            .localization_resources(vec!["builtin.ftl".to_string()], basedir)
            .configure_env(ui_theme::initialize_theme);
    }

    #[instrument(skip_all)]
    pub fn create_initial_ui_state(&self) -> UIState {
        let initial_ui_state = UIState {
            url: self.url.to_string(),
            selected_browser: "".to_string(),
            focused_index: None,
            incognito_mode: false,
            browsers: self.ui_browsers.clone(),
            filtered_browsers: self.filtered_browsers.clone(),
            restorable_app_profiles: self.restorable_app_profiles.clone(),
            show_set_as_default: self.show_set_as_default,
            ui_settings: self.ui_settings.clone(),
            has_non_main_window_open: false,
        };
        return initial_ui_state;
    }

    pub fn print_visible_options(&self) {
        println!("BROWSERS");
        println!();

        for ui_browser in self.filtered_browsers.iter() {
            println!("{}", ui_browser.get_full_name())
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct UIState {
    pub(crate) url: String,
    selected_browser: String,
    focused_index: Option<usize>,
    incognito_mode: bool,

    browsers: Arc<Vec<UIBrowser>>,

    // same as browsers, but filtered view - only the ones matching current url
    filtered_browsers: Arc<Vec<UIBrowser>>,
    pub(crate) restorable_app_profiles: Arc<Vec<UIBrowser>>,

    pub(crate) show_set_as_default: bool,

    pub ui_settings: UISettings,

    // Has About or Settings dialog or a context menu open (e.g right click or 3-dot menu)
    pub has_non_main_window_open: bool,
}

#[derive(Clone, Data, Lens)]
pub struct UISettings {
    pub tab: SettingsTab,
    pub default_opener: Option<UIProfileAndIncognito>,
    pub rules: Arc<Vec<UISettingsRule>>,
    pub visual_settings: UIVisualSettings,
    pub behavioral_settings: UIBehavioralSettings,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct UIVisualSettings {
    pub show_hotkeys: bool,
    pub quit_on_lost_focus: bool,
    pub theme: ConfiguredTheme,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct UIBehavioralSettings {
    pub unwrap_urls: bool,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct UIProfileAndIncognito {
    pub profile: String,
    pub incognito: bool,
}

#[derive(Clone, PartialEq, Data, Copy)]
pub enum SettingsTab {
    GENERAL,
    RULES,
    ADVANCED,
}

impl UISettings {
    pub fn add_empty_rule(&mut self) -> &UISettingsRule {
        info!("add_empty_rule called");

        let next_index = self.rules.len();

        let rule = UISettingsRule {
            index: next_index,
            saved: false,
            deleted: false,
            source_app: "".to_string(),
            url_pattern: "".to_string(),
            opener: None,
        };

        let rules_mut = Arc::make_mut(&mut self.rules);
        rules_mut.push(rule);
        return rules_mut.last().unwrap();
    }

    pub fn mark_rules_as_saved(&mut self) {
        let rules_mut = Arc::make_mut(&mut self.rules);
        for rule in rules_mut.iter_mut() {
            if !rule.deleted {
                rule.saved = true
            }
        }
    }

    /*
    pub fn get_rule_by_index(&self, index: usize) -> Option<&UISettingsRule> {
        return self.rules.get(index);
    }

    pub fn remove_rule_by_index(&mut self, index: usize) {
        let rules_mut = Arc::make_mut(&mut self.rules);
        rules_mut.remove(index);
        // and update .index of all rules
        for (i, rule) in rules_mut.iter_mut().enumerate() {
            rule.index = i
        }
    }*/
}

#[derive(Clone, Debug, Data, Lens)]
pub struct UISettingsRule {
    pub index: usize,
    // useful to recognize if this is a new rule or not,
    // so we can autoscroll to a new rule when it was added
    pub saved: bool,

    // soft-deleting to avoid complex druid issues when reducing array length
    pub deleted: bool,

    // Optional in datamodel
    pub source_app: String,

    // Optional in datamodel
    pub url_pattern: String,

    pub opener: Option<UIProfileAndIncognito>,
}

impl UISettingsRule {
    // converts empty string to None
    pub(crate) fn get_source_app(&self) -> Option<String> {
        return self
            .source_app
            .is_empty()
            .not()
            .then(|| self.source_app.clone());
    }

    // converts empty string to None
    pub(crate) fn get_url_pattern(&self) -> Option<String> {
        return self
            .url_pattern
            .is_empty()
            .not()
            .then(|| self.url_pattern.clone());
    }
}

#[derive(Clone, Data, Lens)]
pub struct UIBrowser {
    // index in not-explicitly-hidden browsers list, used to send message to main event cycle
    // is not impacted by current url, i.e no filters apply
    pub(crate) browser_profile_index: usize,
    pub(crate) is_first: bool,
    pub(crate) is_last: bool,
    restricted_url_matchers: Arc<Vec<UrlGlobMatcher>>,
    pub(crate) browser_name: String,
    pub(crate) profile_name: String,
    profile_name_maybe: Option<String>,
    pub(crate) supports_profiles: bool,
    pub(crate) supports_incognito: bool,

    icon_path: String,
    profile_icon_path: String,
    pub unique_id: String,
    pub(crate) unique_app_id: String,

    // index in list of actually visible browsers for current url
    // (correctly set only in filtered_browsers list)
    pub(crate) filtered_index: usize,
}

impl UIBrowser {
    pub fn has_priority_ordering(&self) -> bool {
        return !self.restricted_url_matchers.is_empty();
    }

    /// Returns app name + optionally profile name if app supports multiple profiles
    pub fn get_full_name(&self) -> String {
        return if self.supports_profiles {
            format!(
                "{} {}",
                self.browser_name.to_string(),
                self.profile_name.as_str()
            )
        } else {
            self.browser_name.to_string()
        };
    }
}

impl UIState {}

// "url_opened" is automatically triggered in macOS
pub const URL_OPENED: Selector<druid::UrlOpenInfo> = Selector::new("url_opened");

// fixed_url_opened is always triggered by Browsers
pub const FIXED_URL_OPENED: Selector<druid::UrlOpenInfo> = Selector::new("fixed_url_opened");
pub const APP_LOST_FOCUS: Selector<druid::ApplicationLostFocus> = Selector::new("app_lost_focus");

pub const EXIT_APP: Selector<String> = Selector::new("browsers.exit_app");

pub const OPEN_LINK_IN_BROWSER_COMPLETED: Selector<String> =
    Selector::new("browsers.open_link_completed");

pub const NEW_BROWSERS_RECEIVED: Selector<Vec<UIBrowser>> =
    Selector::new("browsers.new_browsers_received");

pub const NEW_HIDDEN_BROWSERS_RECEIVED: Selector<Vec<UIBrowser>> =
    Selector::new("browsers.new_hidden_browsers_received");

// or save draft?
// or save rules, but allow "invalid" rules to be saved and handle them?
pub const SAVE_RULES: Selector<()> = Selector::new("browsers.save_rules");
pub const SAVE_RULE: Selector<usize> = Selector::new("browsers.save_rule");
pub const SAVE_DEFAULT_RULE: Selector<()> = Selector::new("browsers.save_default_rule");
pub const SAVE_UI_SETTINGS: Selector<()> = Selector::new("browsers.save_ui_settings");
pub const SAVE_BEHAVIORAL_SETTINGS: Selector<()> =
    Selector::new("browsers.save_behavioral_settings");

pub struct UIDelegate {
    main_sender: Sender<MessageToMain>,
    main_window_id: WindowId,
    windows: Vec<WindowId>,
    mouse_position: Point,
    monitor: Monitor,
}

impl UIDelegate {
    fn save_config_rules(&self, rules_arc: &Arc<Vec<UISettingsRule>>) {
        let rules_clone = rules_arc.clone();
        let rules_vec: Vec<UISettingsRule> = rules_clone
            .iter()
            .filter(|r| !r.deleted)
            .map(|a| a.clone())
            .collect();

        self.main_sender
            .send(MessageToMain::SaveConfigRules(rules_vec))
            .ok();
    }

    fn save_config_default_opener(&self, default_opener: &Option<UIProfileAndIncognito>) {
        self.main_sender
            .send(MessageToMain::SaveConfigDefaultOpener(default_opener.clone()))
            .ok();
    }

    fn save_ui_settings(&self, ui_settings: &UIVisualSettings) {
        self.main_sender
            .send(MessageToMain::SaveConfigUISettings(ui_settings.clone()))
            .ok();
    }

    fn save_behavioral_settings(&self, ui_behavioral_settings: &UIBehavioralSettings) {
        self.main_sender
            .send(MessageToMain::SaveConfigUIBehavioralSettings(
                ui_behavioral_settings.clone(),
            ))
            .ok();
    }

    fn open_link_in_filtered_browser(
        &self,
        ctx: &mut DelegateCtx,
        data: &mut UIState,
        filtered_profile_index: usize,
    ) {
        let browser_index_maybe = data
            .filtered_browsers
            .get(filtered_profile_index)
            .map(|b| b.browser_profile_index);

        if browser_index_maybe.is_some() {
            let browser_index = browser_index_maybe.unwrap();
            ctx.get_external_handle()
                .submit_command(OPEN_LINK_IN_BROWSER, browser_index, Target::Global)
                .ok();
        }
    }
}

impl AppDelegate<UIState> for UIDelegate {
    // move event handling to main window
    fn event(
        &mut self,
        ctx: &mut DelegateCtx,
        window_id: WindowId,
        event: Event,
        data: &mut UIState,
        _env: &Env,
    ) -> Option<Event> {
        if window_id != self.main_window_id {
            // another window triggered event (e.g About or Settings) - ignore
            return Some(event);
        }

        let is_mac = cfg!(target_os = "macos");
        // linux calls this even when just opening a context menu
        // mac calls this when opening About window
        // mac is handled by application event instead now, which is fired
        // when all windows of app loose focus
        let quit_on_lost_focus = !is_mac && data.ui_settings.visual_settings.quit_on_lost_focus;

        let should_exit = match event {
            Event::KeyDown(KeyEvent {
                key: KbKey::Escape, ..
            }) => true,
            Event::WindowLostFocus => quit_on_lost_focus,
            _ => false,
        };

        if should_exit {
            let sink = ctx.get_external_handle();
            // ctx.send_command() does not work correctly on WindowLostFocus
            sink.submit_command(EXIT_APP, "".to_string(), Target::Global)
                .unwrap();
            return None;
        }

        // Cmd+C on macOS, Ctrl+C on windows/linux/OpenBSD
        /*
        let copy_hotkey = HotKey::new(SysMods::Cmd, "c");

        match event {
            Event::KeyDown(keyEvent) => {
                copy_hotkey.matches(keyEvent)

                debug!("Enter caught in delegate");
                if let Some(focused_index) = data.focused_index {
                    ctx.get_external_handle()
                        .submit_command(OPEN_LINK_IN_BROWSER, focused_index, Target::Global)
                        .ok();
                }
            }
        }*/

        // Cmd+C on macOS, Ctrl+C on windows/linux/OpenBSD
        #[cfg(target_os = "macos")]
        let copy_key_mod = Modifiers::META;

        #[cfg(not(target_os = "macos"))]
        let copy_key_mod = Modifiers::CONTROL;

        match event {
            Event::KeyDown(KeyEvent {
                key: KbKey::Character(ref key),
                ref mods,
                ..
            }) if key == "c" && mods == &copy_key_mod => {
                debug!("Cmd/Ctrl+C caught in delegate");
                ctx.get_external_handle()
                    .submit_command(COPY_LINK_TO_CLIPBOARD, {}, Target::Global)
                    .ok();
            }

            Event::KeyDown(KeyEvent {
                key: KbKey::Character(ref key),
                ref mods,
                ..
            }) if key == "," && mods == &copy_key_mod => {
                debug!("Cmd/Ctrl+, caught in delegate");
                ctx.get_external_handle()
                    .submit_command(SHOW_SETTINGS_DIALOG, {}, Target::Global)
                    .ok();
            }

            Event::KeyDown(KeyEvent { code, .. }) => match code {
                Code::Space | Code::Enter => {
                    if let Some(focused_index) = data.focused_index {
                        ctx.get_external_handle()
                            .submit_command(OPEN_LINK_IN_BROWSER, focused_index, Target::Global)
                            .ok();
                    }
                }
                Code::ShiftLeft | Code::ShiftRight => {
                    data.incognito_mode = true;
                }
                Code::Digit1 | Code::Numpad1 => self.open_link_in_filtered_browser(ctx, data, 0),
                Code::Digit2 | Code::Numpad2 => self.open_link_in_filtered_browser(ctx, data, 1),
                Code::Digit3 | Code::Numpad3 => self.open_link_in_filtered_browser(ctx, data, 2),
                Code::Digit4 | Code::Numpad4 => self.open_link_in_filtered_browser(ctx, data, 3),
                Code::Digit5 | Code::Numpad5 => self.open_link_in_filtered_browser(ctx, data, 4),
                Code::Digit6 | Code::Numpad6 => self.open_link_in_filtered_browser(ctx, data, 5),
                Code::Digit7 | Code::Numpad7 => self.open_link_in_filtered_browser(ctx, data, 6),
                Code::Digit8 | Code::Numpad8 => self.open_link_in_filtered_browser(ctx, data, 7),
                Code::Digit9 | Code::Numpad9 => self.open_link_in_filtered_browser(ctx, data, 8),
                Code::Digit0 | Code::Numpad0 => self.open_link_in_filtered_browser(ctx, data, 9),
                _ => {}
            },

            Event::KeyUp(KeyEvent { code, .. }) => match code {
                Code::ShiftLeft | Code::ShiftRight => {
                    data.incognito_mode = false;
                }
                _ => {}
            },

            _ => {}
        }

        // println!("{:?}", event);

        Some(event)
    }

    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut UIState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(EXIT_APP) {
            info!("Exiting Browsers");
            ctx.submit_command(QUIT_APP);
            // QUIT_APP doesn't always actually quit the app on macOS, so forcing exit until thats figured out
            exit(0x0100);
            // Handled::Yes
        } else if cmd.is(APP_LOST_FOCUS) {
            info!("App lost focus");
            if data.ui_settings.visual_settings.quit_on_lost_focus {
                if !data.has_non_main_window_open {
                    let sink = ctx.get_external_handle();
                    sink.submit_command(EXIT_APP, "".to_string(), Target::Global)
                        .unwrap();
                }
            }
            Handled::Yes
        } else if cmd.is(URL_OPENED) {
            let url_open_info = cmd.get_unchecked(URL_OPENED);

            let settings = &data.ui_settings.behavioral_settings;
            let behavioral_config = BehavioralConfig {
                unwrap_urls: settings.unwrap_urls,
            };

            self.main_sender
                .send(MessageToMain::UrlPassedToMain(
                    url_open_info.source_bundle_id.clone(),
                    url_open_info.url.clone(),
                    behavioral_config,
                ))
                .ok();
            Handled::Yes
        } else if cmd.is(FIXED_URL_OPENED) {
            let url_open_info = cmd.get_unchecked(FIXED_URL_OPENED);
            data.url = url_open_info.url.clone();

            let filtered_browsers = get_filtered_browsers(&data.url, &data.browsers);
            data.filtered_browsers = Arc::new(filtered_browsers);

            let (mouse_position, monitor) = druid::Screen::get_mouse_position();
            self.mouse_position = mouse_position;
            self.monitor = monitor;

            let screen_rect = &self
                .monitor
                .virtual_work_rect()
                // add some spacing around screen
                .inflate(-5f64, -5f64);

            let browser_count = (&data.filtered_browsers).len();
            let window_size = recalculate_window_size(browser_count);
            let window_position =
                calculate_window_position(&self.mouse_position, &screen_rect, &window_size);

            // Immediately update window position (so it appears where user clicked).
            let sink = ctx.get_external_handle();
            let target_window = Target::Window(self.main_window_id);
            sink.submit_command(
                CONFIGURE_WINDOW_SIZE_AND_POSITION,
                (window_size, window_position),
                target_window,
            )
            .unwrap();

            // After current event has been handled, bring the window to the front, and give it focus.
            // Normally not needed, but if About menu was opened, then window would not have appeared
            ctx.submit_command(SHOW_WINDOW.to(target_window));

            self.main_sender
                .send(MessageToMain::LinkOpenedFromBundle(
                    url_open_info.source_bundle_id.clone(),
                    url_open_info.url.clone(),
                ))
                .ok();
            Handled::Yes
        } else if cmd.is(SET_FOCUSED_INDEX) {
            let profile_index = cmd.get_unchecked(SET_FOCUSED_INDEX);
            data.focused_index = profile_index.clone();
            Handled::Yes
        } else if cmd.is(COPY_LINK_TO_CLIPBOARD) {
            copy_to_clipboard(data.url.as_str());
            Handled::Yes
        } else if cmd.is(OPEN_LINK_IN_BROWSER) {
            let profile_index = cmd.get_unchecked(OPEN_LINK_IN_BROWSER);
            self.main_sender
                .send(MessageToMain::OpenLink(
                    *profile_index,
                    data.incognito_mode,
                    data.url.to_string(),
                ))
                .ok();
            Handled::Yes
        } else if cmd.is(OPEN_LINK_IN_BROWSER_COMPLETED) {
            let sink = ctx.get_external_handle();
            sink.submit_command(EXIT_APP, "".to_string(), Target::Global)
                .unwrap();
            Handled::Yes
        } else if cmd.is(REFRESH) {
            self.main_sender.send(MessageToMain::Refresh).ok();
            Handled::Yes
        } else if cmd.is(NEW_BROWSERS_RECEIVED) {
            let ui_browsers = cmd.get_unchecked(NEW_BROWSERS_RECEIVED).clone();
            // let old_v = std::mem::replace(&mut data.browsers, Arc::new(ui_browsers));
            data.browsers = Arc::new(ui_browsers);
            let filtered_browsers = get_filtered_browsers(&data.url, &data.browsers);
            data.filtered_browsers = Arc::new(filtered_browsers);

            let mouse_position = self.mouse_position;

            let screen_rect = self
                .monitor
                .virtual_work_rect()
                // add some spacing around screen
                .inflate(-5f64, -5f64);

            let browser_count = (&data.filtered_browsers).len();
            let window_size = recalculate_window_size(browser_count);
            let window_position =
                calculate_window_position(&mouse_position, &screen_rect, &window_size);

            // Immediately update window position (so it appears where user clicked).
            let sink = ctx.get_external_handle();
            let target_window = Target::Window(self.main_window_id);
            sink.submit_command(
                CONFIGURE_WINDOW_SIZE_AND_POSITION,
                (window_size, window_position),
                target_window,
            )
            .unwrap();

            Handled::Yes
        } else if cmd.is(NEW_HIDDEN_BROWSERS_RECEIVED) {
            let restorable_app_profiles = cmd.get_unchecked(NEW_HIDDEN_BROWSERS_RECEIVED).clone();
            // let old_v = std::mem::replace(&mut data.browsers, Arc::new(ui_browsers));
            data.restorable_app_profiles = Arc::new(restorable_app_profiles);
            Handled::Yes
        } else if cmd.is(SET_BROWSERS_AS_DEFAULT_BROWSER) {
            self.main_sender
                .send(MessageToMain::SetBrowsersAsDefaultBrowser)
                .ok();
            Handled::Yes
        } else if cmd.is(HIDE_ALL_PROFILES) {
            let hideable_app_id = cmd.get_unchecked(HIDE_ALL_PROFILES);
            let app_id = hideable_app_id.clone();
            self.main_sender
                .send(MessageToMain::HideAllProfiles(app_id))
                .ok();
            Handled::Yes
        } else if cmd.is(HIDE_PROFILE) {
            let hideable_app_profile_id = cmd.get_unchecked(HIDE_PROFILE);
            let unique_id = hideable_app_profile_id.clone();
            self.main_sender
                .send(MessageToMain::HideAppProfile(unique_id))
                .ok();
            Handled::Yes
        } else if cmd.is(RESTORE_HIDDEN_PROFILE) {
            let restorable_app_profile_id = cmd.get_unchecked(RESTORE_HIDDEN_PROFILE);
            let unique_id = restorable_app_profile_id.clone();
            self.main_sender
                .send(MessageToMain::RestoreAppProfile(unique_id))
                .ok();
            Handled::Yes
        } else if cmd.is(MOVE_PROFILE) {
            let (unique_id, move_to) = cmd.get_unchecked(MOVE_PROFILE);
            let unique_id = unique_id.clone();
            self.main_sender
                .send(MessageToMain::MoveAppProfile(unique_id, move_to.clone()))
                .ok();
            Handled::Yes
        } else if cmd.is(SHOW_ABOUT_DIALOG) {
            about_dialog::show_about_dialog(ctx, self.monitor.clone());
            Handled::Yes
        } else if cmd.is(SHOW_SETTINGS_DIALOG) {
            settings_window::show_settings_dialog(ctx, self.monitor.clone(), &data.browsers);
            Handled::Yes
        } else if cmd.is(SAVE_RULES) {
            self.save_config_rules(&data.ui_settings.rules);
            data.ui_settings.mark_rules_as_saved();
            Handled::Yes
        } else if cmd.is(SAVE_RULE) {
            self.save_config_rules(&data.ui_settings.rules);
            data.ui_settings.mark_rules_as_saved();
            Handled::Yes
        } else if cmd.is(SAVE_DEFAULT_RULE) {
            self.save_config_default_opener(&data.ui_settings.default_opener);
            Handled::Yes
        } else if cmd.is(SAVE_UI_SETTINGS) {
            self.save_ui_settings(&data.ui_settings.visual_settings);
            Handled::Yes
        } else if cmd.is(SAVE_BEHAVIORAL_SETTINGS) {
            self.save_behavioral_settings(&data.ui_settings.behavioral_settings);
            Handled::Yes
        } else {
            //println!("cmd forwarded: {:?}", cmd);
            Handled::No
        }
    }

    fn window_added(
        &mut self,
        id: WindowId,
        _handle: WindowHandle,
        data: &mut UIState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        debug!("Window added, id: {:?}", id);
        self.windows.push(id);
        data.has_non_main_window_open = self.has_non_main_window_open()
    }

    fn window_removed(
        &mut self,
        id: WindowId,
        data: &mut UIState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        debug!("Window removed, id: {:?}", id);
        if let Some(pos) = self.windows.iter().position(|x| *x == id) {
            self.windows.remove(pos);
            data.has_non_main_window_open = self.has_non_main_window_open()
        }
    }
}

impl UIDelegate {
    fn has_non_main_window_open(&self) -> bool {
        return self.windows.iter().any(|x| *x != self.main_window_id);
    }
}

pub(crate) fn get_filtered_browsers(
    url: &str,
    ui_browsers: &Arc<Vec<UIBrowser>>,
) -> Vec<UIBrowser> {
    let url_maybe = Url::parse(url).ok();

    let mut filtered: Vec<UIBrowser> = ui_browsers
        .iter()
        .cloned()
        .filter(|b| {
            return if b.restricted_url_matchers.is_empty() {
                true
            } else {
                url_maybe
                    .as_ref()
                    .map(|url| {
                        let restricted_hostname_matchers = &b.restricted_url_matchers;
                        restricted_hostname_matchers
                            .iter()
                            .any(|matcher| matcher.url_matches(url))
                    })
                    .unwrap_or(false)
            };
        })
        .enumerate()
        .map(|(index, mut browser)| {
            browser.filtered_index = index;
            browser
        })
        .collect();

    // always show special apps first
    filtered.sort_by_key(|b| !b.has_priority_ordering());

    return filtered;
}

fn copy_to_clipboard(url: &str) {
    let mut clipboard = Application::global().clipboard();
    clipboard.put_string(url);
}
