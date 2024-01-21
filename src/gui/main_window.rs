use std::cmp;
use std::sync::Arc;

use druid::piet::InterpolationMode;
use druid::widget::{
    Container, Controller, ControllerHost, CrossAxisAlignment, Either, Flex, Image, Label,
    LineBreaking, List, ZStack,
};
use druid::{
    Color, Env, Event, EventCtx, FontDescriptor, FontFamily, FontWeight, ImageBuf, Lens, LensExt,
    LocalizedString, Menu, MenuItem, Monitor, Point, Rect, RenderContext, Selector, Size, Target,
    TextAlignment, UnitPoint, Vec2, Widget, WidgetExt, WindowDesc, WindowInitialPosition,
    WindowLevel, WindowSizePolicy,
};
use tracing::{debug, instrument};

use crate::gui::focus_widget::{FocusData, FocusWidget};
use crate::gui::image_controller::UIImageController;
use crate::gui::shared;
use crate::gui::ui::{UIBrowser, UIState, EXIT_APP};
use crate::gui::ui_util::ellipsize;
use crate::MoveTo;

pub const COPY_LINK_TO_CLIPBOARD: Selector<()> = Selector::new("browsers.copy_link");

pub const REFRESH: Selector<()> = Selector::new("browsers.refresh");

pub const SET_FOCUSED_INDEX: Selector<Option<usize>> = Selector::new("browsers.hover");

pub const SHOW_ABOUT_DIALOG: Selector<()> = Selector::new("browsers.show_about_dialog");
pub const SHOW_SETTINGS_DIALOG: Selector<()> = Selector::new("browsers.show_settings_dialog");

pub const SET_BROWSERS_AS_DEFAULT_BROWSER: Selector<()> =
    Selector::new("browsers.set-browsers-as-default-browser");

// command to open a link in a selected web browser profile (browser profile index sent via command)
pub const OPEN_LINK_IN_BROWSER: Selector<usize> = Selector::new("browsers.open_link");

pub const HIDE_PROFILE: Selector<String> = Selector::new("browsers.hide_profile");

pub const HIDE_ALL_PROFILES: Selector<String> = Selector::new("browsers.hide_all_profiles");

pub const RESTORE_HIDDEN_PROFILE: Selector<String> =
    Selector::new("browsers.restore_hidden_profile");

pub const MOVE_PROFILE: Selector<(String, MoveTo)> = Selector::new("browsers.move_profile");

const WINDOW_BORDER_WIDTH: f64 = 1.0;
const PADDING_X: f64 = 5.0;
const PADDING_Y: f64 = 10.0;
const ITEM_WIDTH: f64 = 210.0;
const ITEM_HEIGHT: f64 = 32.0;

impl FocusData for UIState {
    fn has_autofocus(&self) -> bool {
        return false;
    }
}
// need to implement this for the Widget<(bool, UIBrowser)> types we declared
impl FocusData for (bool, UIBrowser) {
    fn has_autofocus(&self) -> bool {
        let browser = &self.1;
        return browser.filtered_index == 0;
    }
}

pub struct MainWindow {
    filtered_browsers: Arc<Vec<UIBrowser>>,
    show_set_as_default: bool,
    show_hotkeys: bool,
    show_settings: bool,
}

impl MainWindow {
    pub fn new(
        filtered_browsers: Arc<Vec<UIBrowser>>,
        show_set_as_default: bool,
        show_hotkeys: bool,
        show_settings: bool,
    ) -> Self {
        Self {
            filtered_browsers: filtered_browsers,
            show_set_as_default: show_set_as_default,
            show_hotkeys: show_hotkeys,
            show_settings: show_settings,
        }
    }

    pub fn create_main_window(
        self,
        mouse_position: &Point,
        monitor: &Monitor,
    ) -> WindowDesc<UIState> {
        //let (mouse_position, monitor) = druid::Screen::get_mouse_position();
        let screen_rect = monitor
            .virtual_work_rect()
            // add some spacing around screen
            .inflate(-5f64, -5f64);

        let window_size = recalculate_window_size(&self.filtered_browsers);
        let window_position =
            calculate_window_position(&mouse_position, &screen_rect, &window_size);

        let main_window = WindowDesc::new(self.ui_builder(window_size))
            .show_titlebar(false)
            .skip_taskbar(true)
            .transparent(true)
            .resizable(false)
            .set_level(WindowLevel::AppWindow)
            .set_always_on_top(true)
            .set_initial_position(WindowInitialPosition::Mouse)
            //.window_size_policy(WindowSizePolicy::Content)
            .window_size_policy(WindowSizePolicy::User)
            .window_size(window_size)
            // .with_min_size() seems to be required on LXDE/OpenBox, or window height is too tall
            .with_min_size((window_size.width, 10.0f64))
            .set_initial_position(WindowInitialPosition::Mouse)
            // fall back to coordinates if backend doesn't support WindowInitialPosition::Mouse
            .set_position(window_position)
            .title("Browsers v".to_owned() + env!("CARGO_PKG_VERSION"));

        return main_window;
    }

    #[instrument(skip_all)]
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
        let show_settings = self.show_settings;

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
                make_options_menu(
                    show_set_as_default,
                    data.restorable_app_profiles.clone(),
                    show_settings,
                ),
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

        let show_hotkeys = self.show_hotkeys;
        let browsers_list =
            List::new(move || create_browser(ImageBuf::empty(), ImageBuf::empty(), show_hotkeys))
                .with_spacing(0.0)
                .lens((UIState::incognito_mode, UIState::filtered_browsers))
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

pub(crate) fn recalculate_window_size(filtered_browsers: &Arc<Vec<UIBrowser>>) -> Size {
    let filtered_browsers_total = filtered_browsers.len();
    let item_count = calculate_visible_browser_count(filtered_browsers_total);
    let window_size = calculate_window_size(item_count);

    debug!(
        "New window height: {}, item count: {}",
        &window_size.height, item_count
    );

    return window_size;
}

fn calculate_window_size(item_count: usize) -> Size {
    let browsers_count_f64 = item_count as f64;
    //let window_width = browsers_count_f64 * (64.0 + 6.0) + PADDING_X * 2.0;
    let window_width = ITEM_WIDTH + PADDING_X * 2.0 + WINDOW_BORDER_WIDTH * 2.0;
    let visible_scroll_area_height = visible_scroll_area_height(browsers_count_f64);
    let window_height = visible_scroll_area_height + 5.0 + 12.0 + PADDING_Y * 2.0 + 10.0;

    let window_size = Size::new(window_width, window_height);
    window_size
}

pub(crate) fn calculate_window_position(
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
    show_hotkeys: bool,
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

    let text_size = 10.0;
    let font = FontDescriptor::new(FontFamily::MONOSPACE)
        .with_weight(FontWeight::NORMAL)
        .with_size(text_size);

    let hotkey_label = Either::new(
        move |(_incognito_mode, item): &(bool, UIBrowser), _env| {
            show_hotkeys && item.filtered_index < 9
        },
        {
            let hotkey_label =
                Label::dynamic(|(_incognito_mode, item): &(bool, UIBrowser), _env: &_| {
                    let hotkey_number = item.filtered_index + 1;
                    let hotkey = hotkey_number.to_string();
                    hotkey
                })
                .with_font(font)
                .with_text_color(Color::from_hex_str("808080").unwrap())
                .fix_size(text_size, text_size)
                .padding(4.0);

            let hotkey_label = Container::new(hotkey_label)
                .background(Color::rgba(0.15, 0.15, 0.15, 1.0))
                .rounded(5.0)
                .border(Color::rgba(0.4, 0.4, 0.4, 0.9), 0.5);

            hotkey_label
        },
        Label::new(""),
    );

    let icon_and_label = Flex::row()
        .with_child(icon_stack)
        .with_child(item_label)
        .with_flex_spacer(1.0)
        .with_child(hotkey_label)
        .with_spacer(15.0);

    let container = Container::new(icon_and_label)
        .fix_size(ITEM_WIDTH, ITEM_HEIGHT)
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

fn make_options_menu(
    show_set_as_default: bool,
    hidden_browsers: Arc<Vec<UIBrowser>>,
    show_settings: bool,
) -> Menu<UIState> {
    let submenu_hidden_apps = shared::restore_apps::make_hidden_apps_menu(hidden_browsers);

    let mut menu = Menu::empty();

    menu = menu.entry(MenuItem::new(LocalizedString::new("Refresh")).on_activate(
        |ctx, _data: &mut UIState, _env| {
            ctx.submit_command(REFRESH);
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

    menu = menu.entry(submenu_hidden_apps);

    if show_settings {
        menu = menu.entry(MenuItem::new(LocalizedString::new("Settings...")).on_activate(
            |ctx, _data: &mut UIState, _env| {
                ctx.submit_command(SHOW_SETTINGS_DIALOG);
            },
        ))
    }

    menu = menu
        .entry(MenuItem::new(LocalizedString::new("About")).on_activate(
            |ctx, _data: &mut UIState, _env| {
                ctx.submit_command(SHOW_ABOUT_DIALOG);
            },
        ))
        .entry(MenuItem::new(LocalizedString::new("Quit")).on_activate(
            |ctx, _data: &mut UIState, _env| {
                ctx.submit_command(EXIT_APP.with("".to_string()));
            },
        ));

    menu
}
