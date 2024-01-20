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
use crate::gui::{about_dialog, main_window, settings_window};
use crate::url_rule::UrlGlobMatcher;
use crate::utils::{Config, UIConfig};
use crate::{CommonBrowserProfile, MessageToMain};

pub struct UI {
    localizations_basedir: PathBuf,
    main_sender: Sender<MessageToMain>,
    url: String,
    ui_browsers: Arc<Vec<UIBrowser>>,
    filtered_browsers: Arc<Vec<UIBrowser>>,
    restorable_app_profiles: Arc<Vec<UIBrowser>>,
    show_set_as_default: bool,
    show_hotkeys: bool,
    quit_on_lost_focus: bool,
    show_settings: bool,
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
                deleted: false,
                source_app: rule
                    .source_app
                    .as_ref()
                    .map_or("".to_string(), |s| s.clone()),
                url_pattern: rule
                    .url_pattern
                    .as_ref()
                    .map_or("".to_string(), |s| s.clone()),
                profile: rule.profile.clone(),
                incognito: rule.incognito,
            })
            .collect();

        return UISettings {
            tab: GENERAL,
            rules: Arc::new(ui_settings_rules),
        };
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
                supports_profiles: p.get_browser_common().supports_profiles(),
                profile_name_maybe: p
                    .get_browser_common()
                    .supports_profiles()
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
        ui_config: &UIConfig,
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
            show_hotkeys: ui_config.show_hotkeys,
            quit_on_lost_focus: ui_config.quit_on_lost_focus,
            show_settings: ui_config.show_settings,
            ui_settings: ui_settings,
        }
    }

    #[instrument(skip_all)]
    pub fn create_app_launcher(self) -> AppLauncher<UIState> {
        let basedir = self.localizations_basedir.to_str().unwrap().to_string();
        let (mouse_position, monitor) = druid::Screen::get_mouse_position();

        let main_window1 = main_window::MainWindow::new(
            self.filtered_browsers.clone(),
            self.show_set_as_default,
            self.show_hotkeys,
            self.show_settings,
        );
        let main_window = main_window1.create_main_window(&mouse_position, &monitor);

        let main_window_id = main_window.id.clone();
        return AppLauncher::with_window(main_window)
            .delegate(UIDelegate {
                main_sender: self.main_sender.clone(),
                windows: vec![main_window_id],
                main_window_id: main_window_id,
                mouse_position: mouse_position.clone(),
                monitor: monitor.clone(),
                quit_on_lost_focus: self.quit_on_lost_focus,
            })
            .localization_resources(vec!["builtin.ftl".to_string()], basedir);
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
            ui_settings: self.ui_settings.clone(),
        };
        return initial_ui_state;
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

    pub ui_settings: UISettings,
}

#[derive(Clone, Data, Lens)]
pub struct UISettings {
    pub tab: SettingsTab,
    pub rules: Arc<Vec<UISettingsRule>>,
}

#[derive(Clone, PartialEq, Data, Copy)]
pub enum SettingsTab {
    GENERAL,
    RULES,
}

impl UISettings {
    pub fn add_empty_rule(&mut self) {
        let next_index = self.rules.len();

        let rule = UISettingsRule {
            index: next_index,
            deleted: false,
            source_app: "".to_string(),
            url_pattern: "".to_string(),
            profile: "".to_string(),
            incognito: false,
        };

        let rules_mut = Arc::make_mut(&mut self.rules);
        rules_mut.push(rule);
        info!("add_rule called")
    }

    /*pub fn get_rule_by_index(&self, index: usize) -> Option<&UISettingsRule> {
        return self.rules.get(index);
    }*/

    /*pub fn remove_rule_by_index(&mut self, index: usize) {
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
    pub deleted: bool, // soft-deleting to avoid complex druid issues

    pub source_app: String,  // Optional in datamodel
    pub url_pattern: String, // Optional in datamodel
    pub profile: String,
    pub incognito: bool,
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
        let mut full_name = self.browser_name.to_string();

        if self.supports_profiles {
            full_name = full_name + " " + self.profile_name.as_str()
        }

        return full_name;
    }
}

impl UIState {}

pub const URL_OPENED: Selector<druid::UrlOpenInfo> = Selector::new("url_opened");
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
pub const REMOVE_RULE: Selector<usize> = Selector::new("browsers.remove_rule");

pub struct UIDelegate {
    main_sender: Sender<MessageToMain>,
    main_window_id: WindowId,
    windows: Vec<WindowId>,
    mouse_position: Point,
    monitor: Monitor,
    quit_on_lost_focus: bool,
}

impl UIDelegate {
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
        let quit_on_lost_focus = !is_mac && self.quit_on_lost_focus;

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

        // TODO: disable if settings dialog is up
        // TODO: disable if main dialog is not focused
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
            #[allow(unreachable_code)]
            Handled::Yes
        } else if cmd.is(APP_LOST_FOCUS) {
            info!("App lost focus");
            if self.quit_on_lost_focus {
                let sink = ctx.get_external_handle();
                sink.submit_command(EXIT_APP, "".to_string(), Target::Global)
                    .unwrap();
            }
            Handled::Yes
        } else if cmd.is(URL_OPENED) {
            let url_open_info = cmd.get_unchecked(URL_OPENED);
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

            let window_size = recalculate_window_size(&data.filtered_browsers);
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

            let window_size = recalculate_window_size(&data.filtered_browsers);
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
            let rules = &data.ui_settings.rules.clone();
            let rules_clone: Vec<UISettingsRule> = rules
                .iter()
                .filter(|r| !r.deleted)
                .map(|a| a.clone())
                .collect();
            self.main_sender
                .send(MessageToMain::SaveConfigRules(rules_clone))
                .ok();
            Handled::Yes
        } else if cmd.is(SAVE_RULE) {
            //let rule_index = cmd.get_unchecked(SAVE_RULE).clone();
            let rules = &data.ui_settings.rules.clone();
            let rules_clone: Vec<UISettingsRule> = rules
                .iter()
                .filter(|r| !r.deleted)
                .map(|a| a.clone())
                .collect();

            //let rule_maybe = data.ui_settings.get_rule_by_index(rule_index);
            //if rule_maybe.is_some() {
            //let rule = rule_maybe.unwrap();
            self.main_sender
                .send(MessageToMain::SaveConfigRules(rules_clone))
                .ok();
            //}
            Handled::Yes
        } else if cmd.is(REMOVE_RULE) {
            let rule_index = cmd.get_unchecked(REMOVE_RULE).clone();

            //data.ui_settings.remove_rule_by_index(rule_index);
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
        _data: &mut UIState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        debug!("Window added, id: {:?}", id);
        self.windows.push(id);
    }

    fn window_removed(
        &mut self,
        id: WindowId,
        _data: &mut UIState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        debug!("Window removed, id: {:?}", id);
        if let Some(pos) = self.windows.iter().position(|x| *x == id) {
            self.windows.remove(pos);
        }
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
