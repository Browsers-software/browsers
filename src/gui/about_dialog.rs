use druid::widget::{Flex, Image, Label};
use druid::{
    DelegateCtx, FontDescriptor, FontFamily, FontWeight, ImageBuf, Monitor, Point, Size, WidgetExt,
    WindowDesc, WindowLevel,
};
use tracing::info;

use crate::gui::ui::UIState;
use crate::gui::ui_theme::AboutWindowTheme;
use crate::gui::{shared, ui_theme};
use crate::paths;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn show_about_dialog(ctx: &mut DelegateCtx, monitor: Monitor) {
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
        Label::new("© 2022-2024 Browsers.software team. \nVisit us at https://browsers.software.")
            .with_text_size(10.0);

    let paths_row = shared::directories_info::directories_info(8.0);

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
        .background(AboutWindowTheme::ENV_WINDOW_BACKGROUND_COLOR)
        .env_scope(|env, data| {
            ui_theme::initialize_theme(env, data);
        });

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
        .with_min_size((size.width, 10.0f64))
        // make sure this window is on top of our main window, so using same window level
        // (except that it doesn't work on macOS)
        .set_level(WindowLevel::AppWindow)
        .show_titlebar(true)
        .resizable(false)
        .set_position(window_position);
    ctx.new_window(new_win);
}
