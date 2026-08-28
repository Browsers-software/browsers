use std::cell::RefCell;
use std::process::exit;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use copypasta::{ClipboardContext, ClipboardProvider};
use slint::language::{DragAction, DropEvent};
use slint::{ComponentHandle, DataTransfer, ModelRc, VecModel, Weak};

use crate::MessageToMain;
use crate::gui::app_state::{UIBrowser, UISettings, get_filtered_browsers};
use crate::gui::generated::{AboutWindow, MainWindow, SettingsWindow};
use crate::gui::icon_loader::IconLoader;
use crate::gui::{about_window, main_window, screen, settings_window, theme};
use crate::{MoveTo, UrlOpenInfo};

pub type SharedState = Rc<RefCell<AppState>>;

pub struct AppState {
    pub main_sender: Sender<MessageToMain>,
    pub main_window: MainWindow,
    pub settings_window: Option<SettingsWindow>,
    pub about_window: Option<AboutWindow>,
    pub icons: IconLoader,

    pub url: String,
    pub browsers: Vec<UIBrowser>,
    pub filtered_browsers: Vec<UIBrowser>,
    pub restorable_app_profiles: Vec<UIBrowser>,
    pub show_set_as_default: bool,
    pub ui_settings: UISettings,
    // number of extra (non-main) windows currently open (Settings/About)
    pub extra_windows_open: u32,
}

#[derive(Clone)]
struct BrowserDragPayload {
    unique_id: String,
    source_index: usize,
}

// This matches MainWindow.delete-drop-area-height. The extra transparent part of the borderless
// window becomes a delete target while a browser row is being dragged.
const DELETE_DROP_AREA_HEIGHT: f32 = 48.0;

impl AppState {
    pub fn quit_on_lost_focus_applies(&self) -> bool {
        self.ui_settings.visual_settings.quit_on_lost_focus && self.extra_windows_open == 0
    }
}

pub fn new(
    main_sender: Sender<MessageToMain>,
    url: String,
    browsers: Vec<UIBrowser>,
    restorable_app_profiles: Vec<UIBrowser>,
    show_set_as_default: bool,
    ui_settings: UISettings,
) -> SharedState {
    let main_window = MainWindow::new().expect("failed to create main window");
    // needs a real VecModel here, not whatever the generated default is - see
    // ui_util::sync_vec_model for why
    main_window.set_browsers(ModelRc::new(VecModel::default()));
    main_window.set_restorable_app_profiles(ModelRc::new(VecModel::default()));
    let filtered_browsers = get_filtered_browsers(&url, &browsers);

    let state = Rc::new(RefCell::new(AppState {
        main_sender,
        main_window,
        settings_window: None,
        about_window: None,
        icons: IconLoader::new(),
        url,
        browsers,
        filtered_browsers,
        restorable_app_profiles,
        show_set_as_default,
        ui_settings,
        extra_windows_open: 0,
    }));

    apply_theme(&state);
    refresh_main_window_model(&state);
    reposition_main_window(&state);
    wire_main_window(&state);

    state
}

pub fn apply_theme(state: &SharedState) {
    let st = state.borrow();
    let system_is_dark = theme::detect_system_is_dark(&st.main_window);
    let palette = theme::resolve_palette(&st.ui_settings.visual_settings, system_is_dark);
    theme::apply_to_main_window(&palette, &st.main_window);
    if let Some(w) = &st.settings_window {
        theme::apply_to_settings_window(&palette, w);
    }
    if let Some(w) = &st.about_window {
        theme::apply_to_about_window(&palette, w);
    }
}

pub fn refresh_main_window_model(state: &SharedState) {
    let mut st = state.borrow_mut();
    let show_hotkeys = st.ui_settings.visual_settings.show_hotkeys;

    let items: Vec<_> = st
        .filtered_browsers
        .clone()
        .iter()
        .map(|b| main_window::to_browser_item(b, &mut st.icons, show_hotkeys))
        .collect();
    let restorable: Vec<_> = st
        .restorable_app_profiles
        .iter()
        .map(main_window::to_restorable_profile)
        .collect();

    let win = st.main_window.clone_strong();
    let show_set_as_default = st.show_set_as_default;
    let url = st.url.clone();
    drop(st);

    let browser_count = items.len() as i32;
    crate::gui::ui_util::sync_vec_model(&win.get_browsers(), items);
    crate::gui::ui_util::sync_vec_model(&win.get_restorable_app_profiles(), restorable);
    win.set_show_set_as_default(show_set_as_default);
    win.set_url_text(crate::gui::ui_util::ellipsize(&url, 28).into());
    win.set_icon_size(main_window::get_icon_size());

    // keep the focus index in range as the (filtered) list changes size - after a hide/restore/move,
    // or the url changing
    let focused = win.get_focused_index();
    if browser_count == 0 {
        win.set_focused_index(0);
    } else if focused < 0 || focused >= browser_count {
        win.set_focused_index(focused.clamp(0, browser_count - 1));
    }
}

// resizes and moves the window to the cursor - only for a genuinely new appearance (initial
// launch, or a new URL opened while already running), not a same-URL refresh of an already-visible
// window
pub fn reposition_main_window(state: &SharedState) {
    let st = state.borrow();
    let browser_count = st.filtered_browsers.len();
    let win = st.main_window.clone_strong();
    drop(st);

    let (mouse, work_area) = screen::mouse_position_and_work_area();
    let dialog_size = main_window::recalculate_window_size(browser_count);
    let size = (dialog_size.0, dialog_size.1 + DELETE_DROP_AREA_HEIGHT);
    let position = main_window::calculate_window_position(mouse, work_area, size);

    // keeps min-size == max-size in main_window.slint, so winit's own resizable-from-constraints
    // logic never re-derives "resizable" from the window's natural (much wider) content bounds
    win.set_pinned_width(size.0);
    win.set_pinned_height(size.1);
    win.window()
        .set_size(slint::LogicalSize::new(size.0, size.1));
    win.window()
        .set_position(slint::LogicalPosition::new(position.x, position.y));
}

// resizes for the current browser count without moving it - for refreshing an already-visible
// window's content (Refresh, restoring a hidden profile)
pub fn resize_main_window(state: &SharedState) {
    let st = state.borrow();
    let browser_count = st.filtered_browsers.len();
    let win = st.main_window.clone_strong();
    drop(st);

    let dialog_size = main_window::recalculate_window_size(browser_count);
    let size = (dialog_size.0, dialog_size.1 + DELETE_DROP_AREA_HEIGHT);
    win.set_pinned_width(size.0);
    win.set_pinned_height(size.1);
    win.window()
        .set_size(slint::LogicalSize::new(size.0, size.1));
}

fn wire_main_window(state: &SharedState) {
    let win = state.borrow().main_window.clone_strong();

    {
        let state = state.clone();
        win.on_open_browser(move |profile_index| {
            open_browser(&state, profile_index as i64);
        });
    }
    {
        let state = state.clone();
        win.on_activate_focused(move || {
            let idx = state.borrow().main_window.get_focused_index();
            let profile_index = state
                .borrow()
                .filtered_browsers
                .get(idx as usize)
                .map(|b| b.browser_profile_index as i64);
            if let Some(profile_index) = profile_index {
                open_browser(&state, profile_index);
            }
        });
    }
    {
        let state = state.clone();
        win.on_open_hotkey(move |hotkey_index| {
            let profile_index = state
                .borrow()
                .filtered_browsers
                .get(hotkey_index as usize)
                .map(|b| b.browser_profile_index as i64);
            if let Some(profile_index) = profile_index {
                open_browser(&state, profile_index);
            }
        });
    }
    {
        let state = state.clone();
        win.on_copy_link(move || {
            let url = state.borrow().url.clone();
            if let Ok(mut ctx) = ClipboardContext::new() {
                let _ = ctx.set_contents(url);
            }
        });
    }
    {
        let state = state.clone();
        win.on_refresh(move || {
            let _ = state.borrow().main_sender.send(MessageToMain::Refresh);
        });
    }
    {
        let state = state.clone();
        win.on_set_browsers_default(move || {
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::SetBrowsersAsDefaultBrowser);
        });
    }
    {
        let state = state.clone();
        win.on_hide_profile(move |id| {
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::HideAppProfile(id.to_string()));
        });
    }
    {
        let state = state.clone();
        win.on_hide_app(move |id| {
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::HideAllProfiles(id.to_string()));
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
        win.on_move_profile(move |id, dir| {
            let move_to = match dir.as_str() {
                "top" => MoveTo::TOP,
                "up" => MoveTo::UP,
                "down" => MoveTo::DOWN,
                _ => MoveTo::BOTTOM,
            };
            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::MoveAppProfile(id.to_string(), move_to));
        });
    }
    win.on_make_browser_drag_data(|id, source_index| {
        let mut transfer = DataTransfer::default();
        transfer.set_user_data(Rc::new(BrowserDragPayload {
            unique_id: id.to_string(),
            source_index: source_index.max(0) as usize,
        }));
        transfer
    });
    win.on_can_drop_browser(|event: DropEvent, _target_index| {
        if event
            .data
            .user_data()
            .and_then(|data| data.downcast::<BrowserDragPayload>().ok())
            .is_some()
        {
            event.proposed_action
        } else {
            DragAction::None
        }
    });
    {
        let state = state.clone();
        win.on_browser_delete_dropped(move |event: DropEvent| {
            if event.proposed_action != DragAction::Move {
                return;
            }

            let Some(payload) = event
                .data
                .user_data()
                .and_then(|data| data.downcast::<BrowserDragPayload>().ok())
            else {
                return;
            };

            let _ = state
                .borrow()
                .main_sender
                .send(MessageToMain::HideAppProfile(payload.unique_id.clone()));
        });
    }
    {
        let state = state.clone();
        win.on_browser_dropped(move |event: DropEvent, target_index| {
            if event.proposed_action != DragAction::Move {
                return;
            }

            let Some(payload) = event
                .data
                .user_data()
                .and_then(|data| data.downcast::<BrowserDragPayload>().ok())
            else {
                return;
            };

            let (sender, browser_count, win) = {
                let st = state.borrow();
                (
                    st.main_sender.clone(),
                    st.filtered_browsers.len(),
                    st.main_window.clone_strong(),
                )
            };
            let source_index = payload.source_index;
            if source_index >= browser_count {
                return;
            }

            let target_index = (target_index.max(0) as usize).min(browser_count);
            if target_index == source_index || target_index == source_index + 1 {
                return;
            }

            let final_index = if target_index > source_index {
                target_index - 1
            } else {
                target_index
            };

            if target_index == 0 {
                let _ = sender.send(MessageToMain::MoveAppProfile(
                    payload.unique_id.clone(),
                    MoveTo::TOP,
                ));
            } else if target_index == browser_count {
                let _ = sender.send(MessageToMain::MoveAppProfile(
                    payload.unique_id.clone(),
                    MoveTo::BOTTOM,
                ));
            } else if target_index < source_index {
                for _ in 0..(source_index - target_index) {
                    let _ = sender.send(MessageToMain::MoveAppProfile(
                        payload.unique_id.clone(),
                        MoveTo::UP,
                    ));
                }
            } else {
                for _ in 0..(target_index - source_index - 1) {
                    let _ = sender.send(MessageToMain::MoveAppProfile(
                        payload.unique_id.clone(),
                        MoveTo::DOWN,
                    ));
                }
            }

            win.set_focused_index(final_index as i32);
        });
    }
    {
        let state = state.clone();
        win.on_show_settings(move || {
            settings_window::open(&state);
        });
    }
    {
        let state = state.clone();
        win.on_show_about(move || {
            about_window::open(&state);
        });
    }
    win.on_quit(move || {
        exit(0x0100);
    });
}

fn open_browser(state: &SharedState, browser_profile_index: i64) {
    if browser_profile_index < 0 {
        return;
    }
    let incognito = state.borrow().main_window.get_incognito_mode();
    let url = state.borrow().url.clone();
    let sender = state.borrow().main_sender.clone();
    let _ = sender.send(MessageToMain::OpenLink(
        browser_profile_index as usize,
        incognito,
        url,
    ));
}

thread_local! {
    // lets UiHandle stay Send without unsafe impl - SharedState itself never crosses threads,
    // only this thread-local handle to it does
    static APP_STATE: RefCell<Option<SharedState>> = const { RefCell::new(None) };
}

// handle the background "main" thread uses to push updates onto the Slint UI thread
#[derive(Clone)]
pub struct UiHandle {
    main_weak: Weak<MainWindow>,
}

impl UiHandle {
    pub fn new(state: &SharedState) -> Self {
        APP_STATE.with(|s| *s.borrow_mut() = Some(state.clone()));
        UiHandle {
            main_weak: state.borrow().main_window.as_weak(),
        }
    }

    fn with_state(&self, f: impl FnOnce(&SharedState) + Send + 'static) {
        let _ = self.main_weak.upgrade_in_event_loop(move |_| {
            APP_STATE.with(|s| {
                if let Some(state) = s.borrow().as_ref() {
                    f(state);
                }
            });
        });
    }

    pub fn new_browsers_received(&self, browsers: Vec<UIBrowser>) {
        self.with_state(move |state| {
            {
                let mut st = state.borrow_mut();
                st.browsers = browsers;
                st.filtered_browsers = get_filtered_browsers(&st.url, &st.browsers);
            }
            refresh_main_window_model(state);
            resize_main_window(state);
        });
    }

    pub fn new_hidden_browsers_received(&self, hidden: Vec<UIBrowser>) {
        self.with_state(move |state| {
            state.borrow_mut().restorable_app_profiles = hidden;
            refresh_main_window_model(state);
        });
    }

    pub fn cleaned_url_opened(&self, info: UrlOpenInfo) {
        self.with_state(move |state| {
            {
                let mut st = state.borrow_mut();
                st.url = info.url;
                st.filtered_browsers = get_filtered_browsers(&st.url, &st.browsers);
            }
            refresh_main_window_model(state);
            reposition_main_window(state);
            let win = state.borrow().main_window.clone_strong();
            win.window().show().ok();
            win.invoke_grab_focus();
        });
    }

    pub fn open_link_completed(&self) {
        self.with_state(|_state| {
            exit(0x0100);
        });
    }
}

pub fn run(state: &SharedState) {
    let win = state.borrow().main_window.clone_strong();
    win.window().show().ok();
    win.invoke_grab_focus();

    {
        use slint::winit_030::WinitWindowAccessor;
        let state = state.clone();
        let win_weak = win.as_weak();
        let _ = slint::spawn_local(async move {
            let Some(win) = win_weak.upgrade() else {
                return;
            };
            let Ok(winit_window) = win.window().winit_window().await else {
                return;
            };

            // re-apply the theme now that the native window actually exists - the eager call in
            // new() runs before the OS dark/light watcher is wired up and can read a stale
            // Palette.color-scheme (saw it launch light even with dark mode + "Match System" on)
            apply_theme(&state);

            // resize-border-width: 0 only removes the cursor hint, doesn't stop OS resizing -
            // dragging this borderless popup hit a real Slint bug (slint-ui/slint#3990), easier to
            // just turn off resizing than fight it
            winit_window.set_resizable(false);
        });
    }

    #[cfg(target_os = "macos")]
    make_main_window_floating(&win);

    #[cfg(target_os = "windows")]
    hide_main_window_from_taskbar(&win);

    slint::run_event_loop().ok();
}

// always-on-top isn't documented for macOS in Slint and doesn't cover per-Space visibility or the
// Windows taskbar either, so both of the functions below go through winit directly instead.
// the native window doesn't exist yet until run_event_loop() starts, so we spawn_local an async
// wait for it instead of polling with a timer

// see macos_native::make_window_floating for what this actually sets and why
#[cfg(target_os = "macos")]
fn make_main_window_floating(win: &MainWindow) {
    use slint::winit_030::{WinitWindowAccessor, winit};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let win_weak = win.as_weak();
    let _ = slint::spawn_local(async move {
        let Some(win) = win_weak.upgrade() else {
            return;
        };
        match win.window().winit_window().await {
            Ok(winit_window) => {
                if let Ok(handle) = winit_window.window_handle()
                    && let RawWindowHandle::AppKit(appkit) = handle.as_raw()
                {
                    crate::macos::macos_native::make_window_floating(appkit.ns_view.as_ptr());
                }
            }
            Err(err) => {
                tracing::warn!(
                    "could not obtain the native window - it will not float above other windows: {err}"
                );
            }
        }
    });
}

// best-effort hides the window from the Windows taskbar - unverified, never actually run on
// Windows from this (macOS) session, see TODO.md
#[cfg(target_os = "windows")]
fn hide_main_window_from_taskbar(win: &MainWindow) {
    use slint::winit_030::{WinitWindowAccessor, winit};
    use winit::platform::windows::WindowExtWindows;

    let win_weak = win.as_weak();
    let _ = slint::spawn_local(async move {
        let Some(win) = win_weak.upgrade() else {
            return;
        };
        match win.window().winit_window().await {
            Ok(winit_window) => winit_window.set_skip_taskbar(true),
            Err(err) => {
                tracing::warn!(
                    "could not obtain the native window - it will still show in the taskbar: {err}"
                );
            }
        }
    });
}
