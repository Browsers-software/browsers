use druid::widget::{ControllerHost, CrossAxisAlignment, Flex, Label, RadioGroup};
use druid::{Lens, LensExt, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::ui::{SAVE_UI_SETTINGS, UISettings, UIState, UIVisualSettings};
use crate::utils::ConfiguredTheme;

pub(crate) fn appearance_content() -> impl Widget<UIState> {
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

    return Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(theme_radio_row);
}
