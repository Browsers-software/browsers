use druid::widget::{CrossAxisAlignment, Flex, Image, Label};
use druid::{
    Color, DelegateCtx, FontDescriptor, FontFamily, FontWeight, ImageBuf, Monitor, Point, Selector,
    Size, WidgetExt, WindowDesc, WindowLevel,
};
use tracing::info;

use crate::gui::ui::UIState;
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
        .set_level(WindowLevel::AppWindow)
        .show_titlebar(true)
        .resizable(false)
        .set_position(window_position);
    ctx.new_window(new_win);
}
