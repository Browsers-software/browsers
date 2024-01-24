use druid::widget::{Button, ControllerHost, CrossAxisAlignment, Flex, Label, Switch};
use druid::{LensExt, TextAlignment, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::shared;
use crate::gui::ui::{UISettings, UIState, UIVisualSettings, SAVE_UI_SETTINGS};

pub(crate) fn general_content() -> impl Widget<UIState> {
    let save_command = SAVE_UI_SETTINGS.with(());

    let hotkeys_switch = ControllerHost::new(
        Switch::new().lens(
            UIState::ui_settings
                .then(UISettings::visual_settings)
                .then(UIVisualSettings::show_hotkeys),
        ),
        rules_view::SaveRulesOnDataChange {
            save_rules_command: save_command.clone(),
        },
    );

    let hotkeys_row = Flex::row()
        .with_child(Label::new("Show Hotkeys"))
        .with_child(hotkeys_switch);

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
        .with_child(hotkeys_row)
        .with_default_spacer()
        .with_child(restore_app_button);
}
