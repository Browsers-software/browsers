use druid::widget::{Button, ControllerHost, CrossAxisAlignment, Flex, Label, Switch};
use druid::{LensExt, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::shared;
use crate::gui::ui::{UISettings, UIState, UIVisualSettings, SAVE_UI_SETTINGS};

pub(crate) fn general_content() -> impl Widget<UIState> {
    const TEXT_SIZE: f64 = 13.0;

    let save_command = SAVE_UI_SETTINGS.with(());

    let hotkeys_switch = ControllerHost::new(
        Switch::new().lens(
            UIState::ui_settings
                .then(UISettings::visual_settings)
                .then(UIVisualSettings::show_hotkeys),
        ),
        rules_view::SubmitCommandOnDataChange {
            command: save_command.clone(),
        },
    );

    let hotkeys_row = Flex::row()
        .with_child(Label::new("Show hotkeys").with_text_size(TEXT_SIZE))
        .with_flex_spacer(1.0)
        .with_child(hotkeys_switch);

    let label = Label::new("Restore App...").with_text_size(TEXT_SIZE);

    let restore_app_button =
        Button::from_label(label).on_click(move |ctx, data: &mut UIState, _env| {
            let hidden_browsers = data.restorable_app_profiles.clone();
            let submenu_hidden_apps = shared::restore_apps::make_hidden_apps_menu(hidden_browsers);
            let point = ctx.window_origin();
            ctx.show_context_menu(submenu_hidden_apps, point);
        });

    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(hotkeys_row)
        .with_default_spacer();

    // Showing this option only for macOS right now,
    // because linux calls this even when just opening a context menu
    // mac is handled by application event instead now, which is fired when all windows of app loose focus
    let is_mac = cfg!(target_os = "macos");
    if is_mac {
        let quit_on_lost_focus_switch = ControllerHost::new(
            Switch::new().lens(
                UIState::ui_settings
                    .then(UISettings::visual_settings)
                    .then(UIVisualSettings::quit_on_lost_focus),
            ),
            rules_view::SubmitCommandOnDataChange {
                command: save_command.clone(),
            },
        );

        let quit_on_lost_focus_row = Flex::row()
            .with_child(Label::new("Quit when focus is lost").with_text_size(TEXT_SIZE))
            .with_flex_spacer(1.0)
            .with_child(quit_on_lost_focus_switch);

        col = col.with_child(quit_on_lost_focus_row).with_default_spacer()
    }

    return col.with_child(restore_app_button);
}
