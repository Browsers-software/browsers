use druid::widget::{
    ControllerHost, CrossAxisAlignment, Flex, Label, RadioGroup, TextBox,
};
use druid::{LensExt, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::ui::{
    UISettings, UIState, UIVisualSettings, SAVE_UI_SETTINGS,
};
use crate::utils::{CustomTheme, ThemeMode};

pub(crate) fn appearance_content() -> impl Widget<UIState> {
    const TEXT_SIZE: f64 = 13.0;
    let save_command = SAVE_UI_SETTINGS.with(());

    let theme_radio_group = ControllerHost::new(
        RadioGroup::column(vec![
            ("Match system", ThemeMode::Auto),
            ("Light", ThemeMode::Light),
            ("Dark", ThemeMode::Dark),
            ("Custom", ThemeMode::Custom),
        ]),
        rules_view::SubmitCommandOnDataChange {
            command: save_command.clone(),
        },
    )
    .lens(
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::theme_mode),
    );

    let theme_radio_row = Flex::row()
        .with_child(Label::new("Theme").with_text_size(TEXT_SIZE))
        .with_flex_spacer(1.0)
        .with_child(theme_radio_group);

    // Custom theme inputs
    let window_bg_input = make_color_input("Window Background", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::window_background),
        save_command.clone());

    let text_color_input = make_color_input("Text Color", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::text_color),
        save_command.clone());
        
    let active_tab_bg_input = make_color_input("Active Tab Background", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::active_tab_background),
        save_command.clone());

    let active_tab_text_input = make_color_input("Active Tab Text", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::active_tab_text),
        save_command.clone());

    let inactive_tab_text_input = make_color_input("Inactive Tab Text", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::inactive_tab_text),
        save_command.clone());

    let hover_background_input = make_color_input("Hover/Selected Background", 
        UIState::ui_settings
            .then(UISettings::visual_settings)
            .then(UIVisualSettings::custom_theme)
            .then(CustomTheme::hover_background),
        save_command.clone());

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(theme_radio_row)
        .with_default_spacer()
        .with_child(Label::new("Custom Theme Colors (Hex)").with_text_size(TEXT_SIZE))
        .with_spacer(5.0)
        .with_child(window_bg_input)
        .with_spacer(5.0)
        .with_child(text_color_input)
        .with_spacer(5.0)
        .with_child(active_tab_bg_input)
        .with_spacer(5.0)
        .with_child(active_tab_text_input)
        .with_spacer(5.0)
        .with_child(inactive_tab_text_input)
        .with_spacer(5.0)
        .with_child(hover_background_input)
}

fn make_color_input(label: &str, lens: impl druid::Lens<UIState, String> + 'static, save_command: druid::Command) -> impl Widget<UIState> {
    let input = ControllerHost::new(
        TextBox::new(),
        rules_view::SubmitCommandOnDataChange {
            command: save_command,
        },
    )
    .lens(lens);

    Flex::row()
        .with_child(Label::new(label).with_text_size(13.0))
        .with_flex_spacer(1.0)
        .with_child(input.fix_width(100.0))
}
