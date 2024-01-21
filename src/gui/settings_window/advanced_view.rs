use druid::widget::{Button, CrossAxisAlignment, Flex, Label};
use druid::{Widget, WidgetExt};

use crate::gui::main_window::{REFRESH, SET_BROWSERS_AS_DEFAULT_BROWSER};
use crate::gui::ui::UISettings;
use crate::paths;

pub(crate) fn advanced_content() -> impl Widget<UISettings> {
    // TODO:
    //  - Set as default button
    //  - Open logs dir
    //  - Open settings dir

    let default_button =
        Button::new("Set Browsers as a Default Browser").on_click(|ctx, _data, _env| {
            ctx.submit_command(SET_BROWSERS_AS_DEFAULT_BROWSER);
        });

    let refresh_apps_button =
        Button::new("Refresh Installed Applications").on_click(|ctx, _data, _env| {
            ctx.submit_command(REFRESH);
        });

    return Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(default_button)
        .with_default_spacer()
        .with_child(refresh_apps_button)
        .with_default_spacer()
        .with_child(directories_info(11.0));
}

fn directories_info<T: druid::Data>(text_size: f64) -> Flex<T> {
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
                .with_child(Label::new("Config").with_text_size(text_size))
                .with_child(Label::new("Cache").with_text_size(text_size))
                .with_child(Label::new("Logs").with_text_size(text_size))
                .with_child(Label::new("Resources").with_text_size(text_size)),
        )
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Label::new(config_root_dir).with_text_size(text_size))
                .with_child(Label::new(cache_root_dir).with_text_size(text_size))
                .with_child(Label::new(logs_root_dir).with_text_size(text_size))
                .with_child(Label::new(resources_root_dir).with_text_size(text_size)),
        );

    return paths_row;
}
