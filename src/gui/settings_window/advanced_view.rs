use druid::widget::{Button, CrossAxisAlignment, Flex, Label};
use druid::Widget;

use crate::gui::main_window::{REFRESH, SET_BROWSERS_AS_DEFAULT_BROWSER};
use crate::gui::shared;
use crate::gui::ui::UIState;

pub(crate) fn advanced_content() -> impl Widget<UIState> {
    let default_button =
        Button::from_label(Label::new("Set Browsers as a Default Browser").with_text_size(14.0))
            .on_click(|ctx, _data, _env| {
                ctx.submit_command(SET_BROWSERS_AS_DEFAULT_BROWSER);
            });

    let refresh_apps_button =
        Button::from_label(Label::new("Refresh Installed Applications").with_text_size(14.0))
            .on_click(|ctx, _data, _env| {
                ctx.submit_command(REFRESH);
            });

    return Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(default_button)
        .with_default_spacer()
        .with_child(refresh_apps_button)
        .with_default_spacer()
        .with_child(Label::new("Directories"))
        .with_default_spacer()
        .with_child(shared::directories_info::directories_info(11.0));
}
