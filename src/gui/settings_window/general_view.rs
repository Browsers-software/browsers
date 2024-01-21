use druid::widget::{Button, CrossAxisAlignment, Flex, Label};
use druid::{TextAlignment, Widget};

use crate::gui::shared;
use crate::gui::ui::UIState;

pub(crate) fn general_content() -> impl Widget<UIState> {
    let label = Label::new("Restore App  ⌄")
        .with_text_size(13.0)
        .with_text_alignment(TextAlignment::Center);

    let restore_app_button =
        Button::from_label(label).on_click(move |ctx, data: &mut UIState, _env| {
            let hidden_browsers = data.restorable_app_profiles.clone();
            let submenu_hidden_apps = shared::restore_apps::make_hidden_apps_menu(hidden_browsers);
            let point = ctx.window_origin();
            ctx.show_context_menu(submenu_hidden_apps, point);
        });

    return Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(restore_app_button);
}
