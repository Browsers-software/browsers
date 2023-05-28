use std::cmp;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use druid::commands::{CONFIGURE_WINDOW_SIZE_AND_POSITION, QUIT_APP, SHOW_ALL, SHOW_WINDOW};
use druid::keyboard_types::Key;
use druid::piet::{InterpolationMode, TextStorage};
use druid::widget::{
    Container, Controller, ControllerHost, CrossAxisAlignment, Either, Flex, Image, Label,
    LineBreaking, List, ZStack,
};
use druid::{
    image, Application, BoxConstraints, FontDescriptor, FontFamily, FontWeight, LayoutCtx, LensExt,
    LifeCycle, LifeCycleCtx, LocalizedString, Menu, MenuItem, Modifiers, Monitor, Rect,
    TextAlignment, UnitPoint, UpdateCtx, Vec2, WidgetId, WindowHandle, WindowLevel,
    WindowSizePolicy,
};
use druid::{
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Event, EventCtx, Handled,
    ImageBuf, KbKey, KeyEvent, Lens, PaintCtx, Point, RenderContext, Selector, Size, Target,
    Widget, WidgetExt, WindowDesc, WindowId,
};
use image::io::Reader as ImageReader;
use tracing::{debug, info};
use url::Url;

use crate::{paths, CommonBrowserProfile, MessageToMain};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const PADDING_X: f64 = 5.0;
const PADDING_Y: f64 = 10.0;
const ITEM_HEIGHT: f64 = 32.0;

pub struct UI {
    localizations_basedir: PathBuf,
    main_sender: Sender<MessageToMain>,
    url: String,
    ui_browsers: Arc<Vec<UIBrowser>>,
    restorable_app_profiles: Arc<Vec<UIBrowser>>,
    show_set_as_default: bool,
}

impl UI {
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
                restricted_domains: Arc::new(p.get_restricted_domains().clone()),
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
    ) -> Self {
        Self {
            localizations_basedir: localizations_basedir,
            main_sender: main_sender.clone(),
            url: url.to_string(),
            ui_browsers: Arc::new(ui_browsers),
            restorable_app_profiles: Arc::new(restorable_app_profiles),
            show_set_as_default: show_set_as_default,
        }
    }

    pub fn create_app_launcher(self) -> AppLauncher<UIState> {
        let basedir = self.localizations_basedir.to_str().unwrap().to_string();

        let (mouse_position, monitor) = druid::Screen::get_mouse_position();
        let screen_rect = monitor
            .virtual_work_rect()
            // add some spacing around screen
            .inflate(-5f64, -5f64);

        let window_size = recalculate_window_size(&self.url, &self.ui_browsers);
        let window_position =
            calculate_window_position(&mouse_position, &screen_rect, &window_size);

        let main_window = WindowDesc::new(self.ui_builder(window_size))
            .show_titlebar(false)
            .transparent(true)
            .resizable(false)
            .set_level(WindowLevel::Utility)
            //.window_size_policy(WindowSizePolicy::Content)
            .window_size_policy(WindowSizePolicy::User)
            .window_size(window_size)
            // .with_min_size() seems to be required on LXDE/OpenBox, or window height is too tall
            .with_min_size((window_size.width, 10.0 as f64))
            .set_position(window_position)
            .title("Browsers v".to_owned() + env!("CARGO_PKG_VERSION"));

        let main_window_id = main_window.id.clone();
        return AppLauncher::with_window(main_window)
            .delegate(UIDelegate {
                main_sender: self.main_sender.clone(),
                windows: vec![main_window_id],
                main_window_id: main_window_id,
                mouse_position: mouse_position.clone(),
                monitor: monitor.clone(),
            })
            .localization_resources(vec!["builtin.ftl".to_string()], basedir);
    }

    pub fn create_initial_ui_state(&self) -> UIState {
        let initial_ui_state = UIState {
            url: self.url.to_string(),
            selected_browser: "".to_string(),
            focused_index: None,
            incognito_mode: false,
            browsers: self.ui_browsers.clone(),
            restorable_app_profiles: self.restorable_app_profiles.clone(),
        };
        return initial_ui_state;
    }

    pub fn ui_builder(&self, window_size: Size) -> impl Widget<UIState> {
        const BOTTOM_ROW_HEIGHT: f64 = 18.0;

        let url_label = Label::dynamic(|data: &UIState, _| ellipsize(data.url.as_str(), 28))
            .with_text_size(12.0)
            .with_text_color(Color::from_hex_str("808080").unwrap())
            .with_line_break_mode(LineBreaking::Clip)
            .with_text_alignment(TextAlignment::Start)
            .fix_height(BOTTOM_ROW_HEIGHT)
            .fix_width(175.0)
            .on_click(move |_ctx, _: &mut UIState, _env| {
                _ctx.get_external_handle()
                    .submit_command(COPY_LINK_TO_CLIPBOARD, {}, Target::Global)
                    .ok();
            });

        const OPTIONS_LABEL_SIZE: f64 = 18.0;

        #[cfg(target_os = "macos")]
        const OPTIONS_LABEL_TEXT_SIZE: f64 = 15.0;

        #[cfg(not(target_os = "macos"))]
        const OPTIONS_LABEL_TEXT_SIZE: f64 = 11.0;

        #[cfg(target_os = "macos")]
        const OPTIONS_LABEL_TEXT_PADDING_TOP: f64 = 4.0;

        #[cfg(not(target_os = "macos"))]
        const OPTIONS_LABEL_TEXT_PADDING_TOP: f64 = 0.0;

        let options_label = Label::new("⋮")
            // with_text_alignment messes up in Windows
            //.with_text_alignment(TextAlignment::Center)
            .with_text_size(OPTIONS_LABEL_TEXT_SIZE)
            .padding((0.0, OPTIONS_LABEL_TEXT_PADDING_TOP, 0.0, 0.0))
            .center()
            .fix_width(OPTIONS_LABEL_SIZE)
            .fix_height(OPTIONS_LABEL_SIZE);

        let show_set_as_default = self.show_set_as_default;
        let options_button = FocusWidget::new(
            options_label,
            |ctx, _data: &UIState, _env| {
                let size = ctx.size();
                let radius = OPTIONS_LABEL_SIZE / 2.0;
                let rounded_rect = size.to_rect().to_rounded_rect(radius);

                //let bounds = ctx.size().to_rect();
                let color = Color::rgba(1.0, 1.0, 1.0, 0.25);
                ctx.fill(rounded_rect, &color);
            },
            |ctx, _data: &UIState, _env| {
                if ctx.has_focus() {
                    ctx.get_external_handle()
                        .submit_command(SET_FOCUSED_INDEX, None, Target::Global)
                        .ok();
                }
            },
        )
        .on_click(move |ctx, data: &mut UIState, _env| {
            // Windows requires exact position relative to the window
            let position = Point::new(
                window_size.width - PADDING_X - OPTIONS_LABEL_SIZE / 2.0,
                window_size.height - PADDING_Y - OPTIONS_LABEL_SIZE / 2.0,
            );

            ctx.show_context_menu(
                make_options_menu(show_set_as_default, data.restorable_app_profiles.clone()),
                position,
            );
        })
        .fix_width(OPTIONS_LABEL_SIZE);

        let bottom_row = Flex::row()
            .with_child(url_label)
            .with_flex_spacer(1.0)
            .with_child(options_button);

        //let x2 = (UIState::incognito_mode, UIState::browsers);
        //let lens = lens!(Arc<Vec<UIBrowser>>, x2);
        //let then1 = lens.map(|a| a.1, |x, y| *x = y);

        //let lens = lens!((bool, f64), 1);

        //let then = lens.map(|x| x / 2.0, |x, y| *x = y * 2.0);
        //let x1 = then.get(&(true, 2.0));
        //assert_eq!(x1, 1.0);

        //LensWrap::new(self, then1);

        let browsers_list = List::new(move || create_browser(ImageBuf::empty(), ImageBuf::empty()))
            .with_spacing(0.0)
            .lens((
                UIState::incognito_mode,
                (UIState::url, UIState::browsers).then(FilteredBrowsersLens),
            ))
            .scroll();

        // viewport size is fixed, while scrollable are is full size
        let browsers_list = Container::new(browsers_list).expand_height();

        let col = Flex::column()
            .with_flex_child(browsers_list, 1.0)
            .with_spacer(5.0)
            .with_child(bottom_row)
            .padding((PADDING_X, PADDING_Y));

        return Container::new(col)
            .background(Color::rgba(0.15, 0.15, 0.15, 0.9))
            .rounded(10.0)
            .border(Color::rgba(0.5, 0.5, 0.5, 0.9), 0.5)
            .expand_height();
    }
}

#[derive(Clone, Data, Lens)]
pub struct UIState {
    url: String,
    selected_browser: String,
    focused_index: Option<usize>,
    incognito_mode: bool,

    browsers: Arc<Vec<UIBrowser>>,
    restorable_app_profiles: Arc<Vec<UIBrowser>>,
}

impl FocusData for UIState {
    fn has_autofocus(&self) -> bool {
        return false;
    }
}
// need to implement this for the Widget<(bool, UIBrowser)> types we declared
impl FocusData for (bool, UIBrowser) {
    fn has_autofocus(&self) -> bool {
        let browser = &self.1;
        return browser.browser_profile_index == 0;
    }
}

#[derive(Clone, Data, Lens)]
pub struct UIBrowser {
    browser_profile_index: usize,
    is_first: bool,
    is_last: bool,
    restricted_domains: Arc<Vec<String>>,
    browser_name: String,
    profile_name: String,
    profile_name_maybe: Option<String>,
    supports_profiles: bool,
    supports_incognito: bool,

    icon_path: String,
    profile_icon_path: String,
    unique_id: String,
    unique_app_id: String,
}

impl UIBrowser {
    pub fn has_priority_ordering(&self) -> bool {
        return !self.restricted_domains.is_empty();
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

pub const EXIT_APP: Selector<String> = Selector::new("browsers.exit_app");

pub const SET_FOCUSED_INDEX: Selector<Option<usize>> = Selector::new("browsers.hover");

// command to open a link in a selected web browser profile (browser profile index sent via command)
pub const OPEN_LINK_IN_BROWSER: Selector<usize> = Selector::new("browsers.open_link");
pub const OPEN_LINK_IN_BROWSER_COMPLETED: Selector<String> =
    Selector::new("browsers.open_link_completed");

pub const COPY_LINK_TO_CLIPBOARD: Selector<()> = Selector::new("browsers.copy_link");

pub const REFRESH: Selector<usize> = Selector::new("browsers.refresh");

pub const NEW_BROWSERS_RECEIVED: Selector<Vec<UIBrowser>> =
    Selector::new("browsers.new_browsers_received");

pub const NEW_HIDDEN_BROWSERS_RECEIVED: Selector<Vec<UIBrowser>> =
    Selector::new("browsers.new_hidden_browsers_received");

pub const SET_BROWSERS_AS_DEFAULT_BROWSER: Selector<()> =
    Selector::new("browsers.set-browsers-as-default-browser");

pub const HIDE_PROFILE: Selector<String> = Selector::new("browsers.hide_profile");

pub const HIDE_ALL_PROFILES: Selector<String> = Selector::new("browsers.hide_all_profiles");

pub const RESTORE_HIDDEN_PROFILE: Selector<String> =
    Selector::new("browsers.restore_hidden_profile");

pub const MOVE_PROFILE: Selector<(String, MoveTo)> = Selector::new("browsers.move_profile");

#[derive(Clone, Copy, Debug)]
pub enum MoveTo {
    UP,
    DOWN,
    TOP,
    BOTTOM,
}
pub const SHOW_ABOUT_DIALOG: Selector<()> = Selector::new("browsers.show_about_dialog");

pub struct UIDelegate {
    main_sender: Sender<MessageToMain>,
    main_window_id: WindowId,
    windows: Vec<WindowId>,
    mouse_position: Point,
    monitor: Monitor,
}

impl AppDelegate<UIState> for UIDelegate {
    fn event(
        &mut self,
        ctx: &mut DelegateCtx,
        _window_id: WindowId,
        event: Event,
        data: &mut UIState,
        _env: &Env,
    ) -> Option<Event> {
        //let is_linux = cfg!(target_os = "linux");
        // linux calls this even when just opening a context menu
        //let close_on_lost_focus = !is_linux;

        // always keep open right now even when focus is lost
        let close_on_lost_focus = false;

        let should_exit = match event {
            Event::KeyDown(KeyEvent {
                key: KbKey::Escape, ..
            }) => true,
            Event::WindowLostFocus => close_on_lost_focus,
            _ => false,
        };

        if should_exit {
            let sink = ctx.get_external_handle();
            // ctx.send_command() does not work correctly on WindowLostFocus
            sink.submit_command(EXIT_APP, "".to_string(), Target::Global)
                .unwrap();
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
                key: KbKey::Enter, ..
            }) => {
                debug!("Enter caught in delegate");
                if let Some(focused_index) = data.focused_index {
                    ctx.get_external_handle()
                        .submit_command(OPEN_LINK_IN_BROWSER, focused_index, Target::Global)
                        .ok();
                }
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::Character(ref char),
                ..
            }) if char == " " => {
                debug!("Space caught in delegate");
                if let Some(focused_index) = data.focused_index {
                    ctx.get_external_handle()
                        .submit_command(OPEN_LINK_IN_BROWSER, focused_index, Target::Global)
                        .ok();
                }
            }
            _ => {}
        }

        match event {
            Event::KeyDown(ref key) => {
                if key.key == Key::Shift {
                    //info!("{:?} pressed", key);
                    data.incognito_mode = true;
                }
            }
            Event::KeyUp(ref key) => {
                if key.key == Key::Shift {
                    //info!("{:?} released ", key);
                    data.incognito_mode = false;
                }
            }
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
            ctx.submit_command(QUIT_APP);
            Handled::Yes
        } else if cmd.is(URL_OPENED) {
            let url_open_info = cmd.get_unchecked(URL_OPENED);
            data.url = url_open_info.url.clone();

            let (mouse_position, monitor) = druid::Screen::get_mouse_position();
            self.mouse_position = mouse_position;
            self.monitor = monitor;

            let screen_rect = &self
                .monitor
                .virtual_work_rect()
                // add some spacing around screen
                .inflate(-5f64, -5f64);

            let window_size = recalculate_window_size(&data.url, &data.browsers);
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
            ctx.submit_command(QUIT_APP);
            Handled::Yes
        } else if cmd.is(REFRESH) {
            self.main_sender.send(MessageToMain::Refresh).ok();
            Handled::Yes
        } else if cmd.is(NEW_BROWSERS_RECEIVED) {
            let ui_browsers = cmd.get_unchecked(NEW_BROWSERS_RECEIVED).clone();
            // let old_v = std::mem::replace(&mut data.browsers, Arc::new(ui_browsers));
            data.browsers = Arc::new(ui_browsers);

            let mouse_position = self.mouse_position;

            let screen_rect = self
                .monitor
                .virtual_work_rect()
                // add some spacing around screen
                .inflate(-5f64, -5f64);

            let window_size = recalculate_window_size(&data.url, &data.browsers);
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
            show_about_dialog(ctx, self.monitor.clone());
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

fn show_about_dialog(ctx: &mut DelegateCtx, monitor: Monitor) {
    info!("Browsers version {}", VERSION);

    let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(14.0);

    let mut buf = ImageBuf::empty();

    let app_icon_path = paths::get_app_icon_path();
    let result = ImageBuf::from_file(app_icon_path);
    if result.is_ok() {
        buf = result.unwrap();
    }
    let image = Image::new(buf).fix_width(64.0).fix_height(64.0);

    let app_icon_row = image;

    let app_name_row: Label<UIState> = Label::new("Browsers")
        .with_text_size(14.0)
        .with_font(font.clone());
    let version_row: Label<UIState> =
        Label::new(format!("Version {}", VERSION)).with_text_size(10.0);

    let copyright_row: Label<UIState> =
        Label::new("© 2022-2023 Browsers.software team. \nVisit us at https://browsers.software.")
            .with_text_size(10.0);

    // .join("") adds trailing "/", indicating for the user that it's a directory
    let config_root_dir = paths::get_config_root_dir().join("");
    let config_root_dir = config_root_dir.as_path().to_str().unwrap().to_string();

    let cache_root_dir = paths::get_cache_root_dir().join("");
    let cache_root_dir = cache_root_dir.as_path().to_str().unwrap().to_string();

    let logs_root_dir = paths::get_logs_root_dir().join("");
    let logs_root_dir = logs_root_dir.as_path().to_str().unwrap().to_string();

    let resources_root_dir = paths::get_resources_basedir().join("");
    let resources_root_dir = resources_root_dir.as_path().to_str().unwrap().to_string();

    let paths_row = Flex::row()
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .with_child(Label::new("Config").with_text_size(8.0))
                .with_child(Label::new("Cache").with_text_size(8.0))
                .with_child(Label::new("Logs").with_text_size(8.0))
                .with_child(Label::new("Resources").with_text_size(8.0)),
        )
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Label::new(config_root_dir).with_text_size(8.0))
                .with_child(Label::new(cache_root_dir).with_text_size(8.0))
                .with_child(Label::new(logs_root_dir).with_text_size(8.0))
                .with_child(Label::new(resources_root_dir).with_text_size(8.0)),
        );

    let col = Flex::column()
        .with_spacer(10.0)
        .with_child(app_icon_row)
        .with_spacer(10.0)
        .with_child(app_name_row)
        .with_spacer(8.0)
        .with_child(version_row)
        .with_spacer(8.0)
        .with_child(copyright_row)
        .with_spacer(6.0)
        .with_child(paths_row)
        .with_flex_spacer(1.0)
        .background(Color::from_hex_str("1b2020").unwrap());

    let size = Size::new(340.0, 260.0);
    //let (_, monitor) = druid::Screen::get_mouse_position();
    let screen_rect = monitor.virtual_work_rect();

    let x = screen_rect.x0 + (screen_rect.x1 - screen_rect.x0) / 2.0 - size.width / 2.0;
    let y = screen_rect.y0 + 190.0;
    let window_position = Point::new(x, y);

    let new_win = WindowDesc::new(col)
        // OpenBox on linux changes title to "Unnamed Window" if it's empty string,
        // so using space instead
        .title(" ")
        .window_size(size)
        // with_min_size helps on LXDE
        .with_min_size((size.width, 10.0 as f64))
        // make sure about dialog is on top of our main window
        // so using same window level
        .set_level(WindowLevel::Utility)
        .show_titlebar(true)
        .resizable(false)
        .set_position(window_position);
    ctx.new_window(new_win);
}

fn calculate_visible_browser_count(browsers_total: usize) -> usize {
    // max 6 items without scrollbar
    let item_count = cmp::min(6, browsers_total);
    // but at least 1 item in case of errors (or window size is too small)
    let item_count = cmp::max(1, item_count);

    item_count
}

fn visible_scroll_area_height(browsers_count_f64: f64) -> f64 {
    let browsers_height = browsers_count_f64 * ITEM_HEIGHT;
    return browsers_height;
}

fn calculate_window_size(item_count: usize) -> Size {
    let browsers_count_f64 = item_count as f64;
    //let window_width = browsers_count_f64 * (64.0 + 6.0) + PADDING_X * 2.0;
    let item_width = 32.0 + 160.0;
    let border_width = 1.0;
    let window_width = item_width + PADDING_X * 2.0 + 2.0 * border_width;
    let visible_scroll_area_height = visible_scroll_area_height(browsers_count_f64);
    let window_height = visible_scroll_area_height + 5.0 + 12.0 + PADDING_Y * 2.0 + 6.0;

    let window_size = Size::new(window_width, window_height);
    window_size
}

fn calculate_window_position(
    mouse_position: &Point,
    screen_rect: &Rect,
    window_size: &Size,
) -> Point {
    let mut x = mouse_position.x;
    let mut y = mouse_position.y;

    let window_width = window_size.width;
    let window_height = window_size.height;

    // if x is less than starting point, start from min starting rect
    if x < screen_rect.x0 {
        x = screen_rect.x0;
    }

    // if it doesn't fit, put it as far as it does fit
    if x + window_width > screen_rect.x1 {
        x = screen_rect.x1 - window_width;
    }

    // if y is less than starting point, start from min starting rect
    if y < screen_rect.y0 {
        y = screen_rect.y0;
    }

    if y + window_height > screen_rect.y1 {
        y = screen_rect.y1 - window_height;
    }

    //let primary_monitor_rect = Self::get_active_monitor_rect();

    // top left corner in a y-down space and with non-negative width and height
    //let origin = primary_monitor_rect.origin();

    // size of the rectangle
    //let display_size = primary_monitor_rect.size();

    //let x = origin.x + (display_size.width - window_size.width) / 2.0;
    //let y = origin.y + (display_size.height - window_size.height) / 2.0;
    return Point::new(x, y);
}

fn make_hidden_apps_menu(hidden_profiles: Arc<Vec<UIBrowser>>) -> Menu<UIState> {
    let mut submenu_hidden_apps = Menu::new(LocalizedString::new("Restore"));

    if !hidden_profiles.is_empty() {
        for hidden_profile in hidden_profiles.iter() {
            let item_name = hidden_profile.get_full_name();
            let profile_unique_id = hidden_profile.unique_id.clone();

            submenu_hidden_apps = submenu_hidden_apps.entry(MenuItem::new(item_name).on_activate(
                move |ctx, _data: &mut UIState, _env| {
                    let command = RESTORE_HIDDEN_PROFILE.with(profile_unique_id.clone());
                    ctx.submit_command(command);
                },
            ));
        }
    } else {
        submenu_hidden_apps =
            submenu_hidden_apps.entry(MenuItem::new("No hidden apps or profiles").enabled(false));
    }

    return submenu_hidden_apps;
}

fn make_options_menu(
    show_set_as_default: bool,
    hidden_browsers: Arc<Vec<UIBrowser>>,
) -> Menu<UIState> {
    let submenu_hidden_apps = make_hidden_apps_menu(hidden_browsers);

    let mut menu = Menu::empty();

    menu = menu.entry(MenuItem::new(LocalizedString::new("Refresh")).on_activate(
        |ctx, _data: &mut UIState, _env| {
            ctx.submit_command(REFRESH.with(0));
        },
    ));

    if show_set_as_default {
        menu = menu.entry(
            MenuItem::new(LocalizedString::new("Make Browsers Default"))
                .on_activate(|ctx, _data: &mut UIState, _env| {
                    ctx.submit_command(SET_BROWSERS_AS_DEFAULT_BROWSER);
                })
                .enabled_if(move |_, _| show_set_as_default),
        );
    }

    menu = menu
        .entry(submenu_hidden_apps)
        .entry(MenuItem::new(LocalizedString::new("About")).on_activate(
            |ctx, _data: &mut UIState, _env| {
                ctx.submit_command(SHOW_ABOUT_DIALOG);
            },
        ))
        .entry(MenuItem::new(LocalizedString::new("Quit")).on_activate(
            |ctx, _data: &mut UIState, _env| {
                ctx.submit_command(QUIT_APP);
            },
        ));

    menu
}

pub struct UIImageController;

impl UIImageController {
    fn get_image_buf(&self, icon_path: &str) -> Result<ImageBuf, Box<dyn Error>> {
        if icon_path.is_empty() {
            return Ok(ImageBuf::empty());
        }

        let path1 = Path::new(icon_path);

        let dynamic_image = ImageReader::open(path1)?.decode()?;
        let buf = ImageBuf::from_dynamic_image(dynamic_image);
        return Ok(buf);
    }
}

impl Controller<String, Image> for UIImageController {
    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        icon_path: &String,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                debug!("WidgetAdded WAS CALLED for icon {}", icon_path.clone());
                if let Ok(buf) = self.get_image_buf(icon_path.as_str()) {
                    child.set_image_data(buf);
                }
            }
            _ => {
                // TODO: check if icon path has changed
                //info!("other event {:?} for icon {}", event, icon_path.clone());
            }
        }

        child.lifecycle(ctx, event, icon_path, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_icon_path: &String,
        icon_path: &String,
        _env: &Env,
    ) {
        if icon_path != old_icon_path {
            debug!(
                "icon changed from {} to {}",
                old_icon_path.clone(),
                icon_path.clone()
            );

            if let Ok(buf) = self.get_image_buf(icon_path.as_str()) {
                child.set_image_data(buf);
                ctx.children_changed();
            }
        }
    }
}

// icon styles are conventionally different on platforms,
// e.g most macos icons are actually with a lot of padding
const fn get_icon_size() -> f64 {
    // 8 + 8; 64/8 = 8
    // 48/8 = 6
    if cfg!(target_os = "macos") {
        32.0
    } else if cfg!(target_os = "linux") {
        24.0
    } else {
        24.0
    }
}

const fn get_icon_padding() -> f64 {
    if cfg!(target_os = "macos") {
        0.0
    } else if cfg!(target_os = "linux") {
        0.0
    } else {
        0.0
    }
}

fn recalculate_window_size(url: &str, ui_browsers: &Arc<Vec<UIBrowser>>) -> Size {
    let filtered_browsers = get_filtered_browsers(url, &ui_browsers);
    let filtered_browsers_total = filtered_browsers.len();
    let item_count = calculate_visible_browser_count(filtered_browsers_total);
    let window_size = calculate_window_size(item_count);

    debug!(
        "New window height: {}, item count: {}",
        &window_size.height, item_count
    );

    return window_size;
}

fn get_filtered_browsers(url: &str, ui_browsers: &Arc<Vec<UIBrowser>>) -> Vec<UIBrowser> {
    let url_result = Url::parse(url);
    let domain_maybe = url_result
        .ok()
        .map(|url| url.host_str().map(|d| d.to_string()))
        .flatten();

    let mut filtered: Vec<UIBrowser> = ui_browsers
        .iter()
        .cloned()
        .filter(|b| {
            if b.restricted_domains.is_empty() {
                return true;
            }

            return if domain_maybe.is_none() {
                false
            } else {
                let domain = domain_maybe.as_ref().unwrap();
                b.restricted_domains.contains(domain)
            };
        })
        .collect();

    // always show special apps first
    filtered.sort_by_key(|b| !b.has_priority_ordering());

    return filtered;
}

/* Filters browsers based on url */
struct FilteredBrowsersLens;

impl FilteredBrowsersLens {
    // gets browsers relevant only for current url
    fn get_filtered_browsers(data: &(String, Arc<Vec<UIBrowser>>)) -> Vec<UIBrowser> {
        return get_filtered_browsers(&data.0, &data.1);
    }
}
impl Lens<(String, Arc<Vec<UIBrowser>>), Arc<Vec<UIBrowser>>> for FilteredBrowsersLens {
    fn with<R, F: FnOnce(&Arc<Vec<UIBrowser>>) -> R>(
        &self,
        data: &(String, Arc<Vec<UIBrowser>>),
        f: F,
    ) -> R {
        let filtered = Self::get_filtered_browsers(data);
        let arc_filtered = Arc::new(filtered);
        f(&arc_filtered)
    }

    fn with_mut<R, F: FnOnce(&mut Arc<Vec<UIBrowser>>) -> R>(
        &self,
        data: &mut (String, Arc<Vec<UIBrowser>>),
        f: F,
    ) -> R {
        let filtered = Self::get_filtered_browsers(data);
        let mut arc_filtered = Arc::new(filtered);
        f(&mut arc_filtered) // &mut data
    }
}

/* Extracts browser from the (bool, UIBrowser) tuple*/
struct BrowserLens;

impl Lens<(bool, UIBrowser), UIBrowser> for BrowserLens {
    fn with<R, F: FnOnce(&UIBrowser) -> R>(&self, data: &(bool, UIBrowser), f: F) -> R {
        f(&data.1)
    }

    fn with_mut<R, F: FnOnce(&mut UIBrowser) -> R>(&self, data: &mut (bool, UIBrowser), f: F) -> R {
        f(&mut data.1)
    }
}

fn create_browser_label() -> Label<(bool, UIBrowser)> {
    let browser_label = Label::dynamic(|(incognito_mode, item): &(bool, UIBrowser), _env| {
        let mut name = item.browser_name.clone();
        if item.supports_incognito && *incognito_mode {
            name += " 👓";
        }
        name
    })
    .with_text_size(12.0)
    .with_line_break_mode(LineBreaking::Clip)
    .with_text_alignment(TextAlignment::Start)
    .with_text_color(Color::from_hex_str("ffffff").unwrap());

    browser_label
}

fn create_browser(
    app_icon_buf: ImageBuf,
    profile_img_buf: ImageBuf,
) -> impl Widget<(bool, UIBrowser)> {
    let icon_size = get_icon_size();
    let icon_padding = get_icon_padding();

    if icon_size + icon_padding * 2.0 > ITEM_HEIGHT {
        // ideally this could be compile time check
        panic!("icon_size + icon_padding > ITEM_HEIGHT");
    }

    let image_widget = Image::new(app_icon_buf)
        .interpolation_mode(InterpolationMode::Bilinear)
        .controller(UIImageController)
        .fix_width(icon_size)
        .fix_height(icon_size)
        .center()
        .padding(icon_padding)
        .lens(BrowserLens.then(UIBrowser::icon_path));

    let profile_icon = Image::new(profile_img_buf.clone())
        .interpolation_mode(InterpolationMode::Bilinear)
        .controller(UIImageController)
        .fix_width(16.0)
        .fix_height(16.0)
        .center()
        .lens(BrowserLens.then(UIBrowser::profile_icon_path));

    let item_label = Either::new(
        |(_incognito_mode, item): &(bool, UIBrowser), _env| item.supports_profiles,
        {
            let profile_label =
                Label::dynamic(|(_incognito_mode, item): &(bool, UIBrowser), _env: &_| {
                    item.profile_name.clone()
                })
                .with_text_size(11.0)
                .with_line_break_mode(LineBreaking::Clip)
                .with_text_alignment(TextAlignment::Start)
                .with_text_color(Color::from_hex_str("BEBEBE").unwrap());

            let profile_row = Flex::row()
                //.with_child(profile_icon)
                .with_child(profile_label);

            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Fill)
                .with_child(create_browser_label())
                .with_child(profile_row)
        },
        {
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Fill)
                .with_child(create_browser_label())
        },
    );

    let icon_stack = ZStack::new(image_widget).with_child(
        profile_icon,
        Vec2::new(1.0, 1.0),
        Vec2::new(16.0, 16.0),
        UnitPoint::new(0.1, 0.1),
        Vec2::ZERO,
    );

    let icon_and_label = Flex::row().with_child(icon_stack).with_child(item_label);

    let container = Container::new(icon_and_label)
        .fix_size(192.0, ITEM_HEIGHT)
        .on_click(move |_ctx, (_, data): &mut (bool, UIBrowser), _env| {
            _ctx.get_external_handle()
                .submit_command(OPEN_LINK_IN_BROWSER, data.browser_profile_index, Target::Global)
                .ok();
        });

    let container = FocusWidget::new(
        container,
        |ctx, _: &(bool, UIBrowser), _env| {
            let size = ctx.size();
            let rounded_rect = size.to_rounded_rect(5.0);
            let color = Color::rgba(1.0, 1.0, 1.0, 0.25);
            ctx.fill(rounded_rect, &color);
        },
        |ctx, (_, data): &(bool, UIBrowser), _env| {
            if ctx.has_focus() {
                ctx.get_external_handle()
                    .submit_command(
                        SET_FOCUSED_INDEX,
                        Some(data.browser_profile_index),
                        Target::Global,
                    )
                    .ok();
            }
        },
    );

    let container = Container::new(container);

    let container = ControllerHost::new(container, ContextMenuController);

    return container;

    // .event(|ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env| {});
    // see https://github.com/linebender/druid/blob/313af5e2cbc3be460dbf9edd609763801ab9190c/druid/src/widget/button.rs#L170
    // draw with hot check
    // re-draw on HotChanged
}

pub trait FocusData {
    fn has_autofocus(&self) -> bool;
}

pub const FOCUS_WIDGET_SET_FOCUS_ON_HOVER: Selector<WidgetId> =
    Selector::new("focus_widget.set_focus");

struct FocusWidget<S: druid::Data + FocusData, W> {
    inner: W,
    paint_fn_on_focus: fn(ctx: &mut PaintCtx, data: &S, env: &Env),
    lifecycle_fn: fn(ctx: &mut LifeCycleCtx, data: &S, env: &Env),
}

impl<S: druid::Data + FocusData, W> FocusWidget<S, W> {}

impl<S: druid::Data + FocusData, W> FocusWidget<S, W> {
    pub fn new(
        inner: W,
        paint_fn_on_focus: fn(ctx: &mut PaintCtx, data: &S, env: &Env),
        lifecycle_fn: fn(ctx: &mut LifeCycleCtx, data: &S, env: &Env),
    ) -> FocusWidget<S, W> {
        FocusWidget {
            inner,
            paint_fn_on_focus,
            lifecycle_fn,
        }
    }
}

impl<S: druid::Data + FocusData, W: Widget<S>> Widget<S> for FocusWidget<S, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut S, env: &Env) {
        match event {
            // on mouse hover request focus
            Event::Command(cmd) if cmd.is(FOCUS_WIDGET_SET_FOCUS_ON_HOVER) => {
                //let widget_id = cmd.get_unchecked(FOCUS_WIDGET_SET_FOCUS_ON_HOVER);
                //info!(
                //    "received FOCUS_WIDGET_SET_FOCUS to widget_id: {:?}",
                //    widget_id
                //);
                ctx.request_focus();
                ctx.request_paint();
                ctx.set_handled();
                ctx.request_update();
            }
            Event::WindowConnected => {
                if data.has_autofocus() {
                    // ask for focus on launch
                    ctx.request_focus();
                }
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::Tab,
                mods,
                ..
            }) => {
                if mods.shift() {
                    debug!("Shift+Tab PRESSED");
                    ctx.focus_prev();
                } else {
                    debug!("Tab PRESSED");
                    ctx.focus_next();
                };

                ctx.request_paint();
                ctx.set_handled();
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::ArrowDown,
                ..
            }) => {
                debug!("ArrowDown PRESSED");

                ctx.focus_next();
                ctx.request_paint();
                ctx.set_handled();
            }
            Event::KeyDown(KeyEvent {
                key: KbKey::ArrowUp,
                ..
            }) => {
                debug!("ArrowUp PRESSED");

                ctx.focus_prev();
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }

        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &S, env: &Env) {
        match event {
            LifeCycle::BuildFocusChain => {
                // widget which can be hovered with a mouse,
                // can also be focused with keyboard navigation
                ctx.register_for_focus();
            }
            LifeCycle::FocusChanged(to_focused) => {
                if *to_focused {
                    // enable scrolling once getting edge cases right
                    // (sometimes too eager to scroll top/bottom item)
                    if !ctx.is_hot() {
                        ctx.scroll_to_view();
                    }
                    (self.lifecycle_fn)(ctx, data, env);
                }
                ctx.request_paint();
            }
            LifeCycle::HotChanged(to_hot) => {
                if *to_hot && !ctx.has_focus() {
                    // when mouse starts "hovering" this item, let's also request focus,
                    // because we consider keyboard navigation and mouse hover the same here
                    let cmd = Command::new(
                        FOCUS_WIDGET_SET_FOCUS_ON_HOVER,
                        ctx.widget_id(),
                        Target::Widget(ctx.widget_id()),
                    );
                    ctx.submit_command(cmd);
                    //ctx.request_paint();
                }
            }
            _ => {}
        }
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &S, data: &S, env: &Env) {
        /*if old_data.glow_hot != data.glow_hot {
            ctx.request_paint();
        }*/
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &S, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &S, env: &Env) {
        if ctx.has_focus() {
            (self.paint_fn_on_focus)(ctx, data, env);
        }
        self.inner.paint(ctx, data, env);
    }
}

struct ContextMenuController;

impl<W: Widget<(bool, UIBrowser)>> Controller<(bool, UIBrowser), W> for ContextMenuController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut (bool, UIBrowser),
        env: &Env,
    ) {
        match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                ctx.show_context_menu(make_context_menu(&data.1), mouse.pos);
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}

fn make_context_menu(browser: &UIBrowser) -> Menu<UIState> {
    let mut menu = Menu::empty();

    let id = browser.unique_id.clone();
    let app_name = browser.browser_name.to_string();

    if !browser.has_priority_ordering() {
        let is_visible = !browser.is_first;
        let item_name = browser.get_full_name();

        let move_profile_to_top_label = LocalizedString::new("move-profile-to-top")
            .with_arg("item-name", move |_, _| item_name.clone().into());

        let this_id = id.clone();
        menu = menu.entry(
            MenuItem::new(move_profile_to_top_label)
                .on_activate(move |ctx, _data: &mut UIState, _env| {
                    let command = MOVE_PROFILE.with((this_id.clone(), MoveTo::TOP));
                    ctx.submit_command(command);
                })
                .enabled_if(move |_, _| is_visible),
        );

        let item_name = browser.get_full_name();
        let move_profile_higher_label = LocalizedString::new("move-profile-higher")
            .with_arg("item-name", move |_, _| item_name.clone().into());

        let this_id = id.clone();
        menu = menu.entry(
            MenuItem::new(move_profile_higher_label)
                .on_activate(move |ctx, _data: &mut UIState, _env| {
                    let command = MOVE_PROFILE.with((this_id.clone(), MoveTo::UP));
                    ctx.submit_command(command);
                })
                .enabled_if(move |_, _| is_visible),
        );

        let is_visible = !browser.is_last;
        let item_name = browser.get_full_name();
        let move_profile_lower_label = LocalizedString::new("move-profile-lower")
            .with_arg("item-name", move |_, _| item_name.to_string().into());

        let this_id = id.clone();
        menu = menu.entry(
            MenuItem::new(move_profile_lower_label)
                .on_activate(move |ctx, _data: &mut UIState, _env| {
                    let command = MOVE_PROFILE.with((this_id.clone(), MoveTo::DOWN));
                    ctx.submit_command(command);
                })
                .enabled_if(move |_, _| is_visible),
        );

        let this_id = id.clone();
        let item_name = browser.get_full_name();
        let move_profile_bottom_label = LocalizedString::new("move-profile-to-bottom")
            .with_arg("item-name", move |_, _| item_name.to_string().into());
        menu = menu.entry(
            MenuItem::new(move_profile_bottom_label)
                .on_activate(move |ctx, _data: &mut UIState, _env| {
                    let command = MOVE_PROFILE.with((this_id.clone(), MoveTo::BOTTOM));
                    ctx.submit_command(command);
                })
                .enabled_if(move |_, _| is_visible),
        );
    }

    let item_name = browser.get_full_name();

    let hide_profile_label = LocalizedString::new("hide-profile")
        .with_arg("item-name", move |_, _| item_name.clone().into());

    let this_id = id.clone();
    menu = menu.entry(MenuItem::new(hide_profile_label).on_activate(
        move |ctx, _data: &mut UIState, _env| {
            let command = HIDE_PROFILE.with(this_id.clone());
            ctx.submit_command(command);
        },
    ));

    if browser.supports_profiles {
        let app_id = browser.unique_app_id.clone();

        let hide_app_label = LocalizedString::new("hide-app")
            .with_arg("app-name", move |_, _| app_name.clone().into());

        menu = menu.entry(MenuItem::new(hide_app_label).on_activate(
            move |ctx, _data: &mut UIState, _env| {
                let command = HIDE_ALL_PROFILES.with(app_id.clone());
                ctx.submit_command(command);
            },
        ));
    }

    menu
}

fn copy_to_clipboard(url: &str) {
    let mut clipboard = Application::global().clipboard();
    clipboard.put_string(url);
}

fn ellipsize(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    text[0..max_length - 1].to_string() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsize_shorter() {
        assert_eq!(ellipsize("some text", 8), "some te…");
    }

    #[test]
    fn test_ellipsize_enough() {
        assert_eq!(ellipsize("some text", 9), "some text");
    }
}
