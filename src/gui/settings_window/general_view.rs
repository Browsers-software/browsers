use druid::widget::{
    Button, ControllerHost, CrossAxisAlignment, Flex, Label, LineBreaking, RadioGroup, Switch,
};
use druid::{LensExt, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::shared;
use crate::gui::ui::{UISettings, UIState, UIVisualSettings, SAVE_UI_SETTINGS};
use crate::utils::ConfiguredTheme;

pub(crate) fn general_content() -> impl Widget<UIState> {
    const TEXT_SIZE: f64 = 13.0;

    let save_command = SAVE_UI_SETTINGS.with(());

    let theme_radio_group = ControllerHost::new(
        RadioGroup::column(vec![
            ("Match system", ConfiguredTheme::Auto),
            ("Light", ConfiguredTheme::Light),
            ("Dark", ConfiguredTheme::Dark),
        ]),
        rules_view::SubmitCommandOnDataChange {
            command: save_command.clone(),
        },
    )
    .lens(
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::theme),
    );

    let theme_radio_row = Flex::row()
        .with_child(Label::new("Theme").with_text_size(TEXT_SIZE))
        .with_flex_spacer(1.0)
        .with_child(theme_radio_group);

    let hotkeys_switch = ControllerHost::new(
        Switch::new(),
        rules_view::SubmitCommandOnDataChange {
            command: save_command.clone(),
        },
    )
    .lens(
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::show_hotkeys),
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
        .with_child(theme_radio_row)
        .with_default_spacer()
        .with_child(hotkeys_row)
        .with_default_spacer();

    // Showing this option only for macOS right now,
    // because linux calls this even when just opening a context menu
    // mac is handled by application event instead now, which is fired when all windows of app loose focus
    let is_mac = cfg!(target_os = "macos");
    if is_mac {
        let quit_on_lost_focus_switch = ControllerHost::new(
            Switch::new(),
            rules_view::SubmitCommandOnDataChange {
                command: save_command.clone(),
            },
        )
        .lens(
            UIState::ui_settings
                .then(UISettings::visual_settings)
                .then(UIVisualSettings::quit_on_lost_focus),
        );

        let quit_on_lost_focus_row = Flex::row()
            .with_child(Label::new("Quit when focus is lost").with_text_size(TEXT_SIZE))
            .with_flex_spacer(1.0)
            .with_child(quit_on_lost_focus_switch);

        col = col.with_child(quit_on_lost_focus_row).with_default_spacer()
    }

    let tooltip = Label::new(
        "To hide and move applications/profiles, close settings and just right-click on the application in the main dialog"
    )
        .with_text_size(11.0)
        .with_line_break_mode(LineBreaking::WordWrap);

    return col
        .with_child(Label::new("Applications and Profiles"))
        .with_default_spacer()
        .with_child(restore_app_button)
        .with_default_spacer()
        .with_child(tooltip);
}
