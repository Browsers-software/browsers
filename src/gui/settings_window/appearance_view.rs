use druid::widget::{ControllerHost, CrossAxisAlignment, Either, Flex, Label, RadioGroup, TextBox};
use druid::{Command, Data, Lens, LensExt, Widget, WidgetExt};

use crate::gui::settings_window::rules_view;
use crate::gui::ui::{SAVE_UI_SETTINGS, UISettings, UIState, UIVisualSettings};
use crate::utils::{ConfiguredTheme, CustomPalette};

pub(crate) fn appearance_content() -> impl Widget<UIState> {
    const TEXT_SIZE: f64 = 13.0;

    let save_command = SAVE_UI_SETTINGS.with(());

    let theme_radio_group = ControllerHost::new(
        RadioGroup::column(vec![
            ("Match system", ConfiguredTheme::Auto),
            ("Light", ConfiguredTheme::Light),
            ("Dark", ConfiguredTheme::Dark),
            ("Custom", ConfiguredTheme::Custom),
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

    let custom_palette_section = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new("Custom Palette").with_text_size(TEXT_SIZE))
        .with_spacer(2.0)
        .with_child(Label::new("Colors are #rrggbb or #rrggbbaa hex.").with_text_size(11.0))
        .with_spacer(8.0)
        .with_child(section_label("General"))
        .with_child(color_row(
            "Background",
            CustomPalette::background,
            save_command.clone(),
        ))
        .with_child(color_row("Label", CustomPalette::label, save_command.clone()))
        .with_child(color_row(
            "Secondary label",
            CustomPalette::secondary_label,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Muted label",
            CustomPalette::muted_label,
            save_command.clone(),
        ))
        .with_child(color_row("Stroke", CustomPalette::stroke, save_command.clone()))
        .with_child(color_row(
            "Secondary stroke",
            CustomPalette::secondary_stroke,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Highlight",
            CustomPalette::highlight,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Secondary background",
            CustomPalette::secondary_background,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Subtle background",
            CustomPalette::subtle_background,
            save_command.clone(),
        ))
        .with_child(color_row("Accent", CustomPalette::accent, save_command.clone()))
        .with_child(color_row(
            "On accent",
            CustomPalette::on_accent,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("Native widgets"))
        .with_child(color_row(
            "Window background",
            CustomPalette::window_background,
            save_command.clone(),
        ))
        .with_child(color_row("Text", CustomPalette::text, save_command.clone()))
        .with_child(color_row(
            "Background (light)",
            CustomPalette::background_light,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Background (dark)",
            CustomPalette::background_dark,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Button (dark)",
            CustomPalette::button_dark,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Button (light)",
            CustomPalette::button_light,
            save_command.clone(),
        ))
        .with_child(color_row("Cursor", CustomPalette::cursor, save_command.clone()))
        .with_child(color_row(
            "About window background",
            CustomPalette::about_background,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("Hover states"))
        .with_child(color_row(
            "Label (hover)",
            CustomPalette::hover_label,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Secondary label (hover)",
            CustomPalette::hover_secondary_label,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey background (hover)",
            CustomPalette::hotkey_hover_background,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey text (hover)",
            CustomPalette::hotkey_hover_text,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("Typography"))
        .with_child(text_row(
            "Font family",
            CustomPalette::font_family,
            save_command.clone(),
        ))
        .with_child(
            Label::new("system-ui, serif, sans-serif, monospace, or a font name")
                .with_text_size(10.0),
        )
        .with_spacer(4.0)
        .with_child(size_row(
            "Browser label size",
            CustomPalette::browser_label_size,
            save_command.clone(),
        ))
        .with_child(size_row(
            "Profile label size",
            CustomPalette::profile_label_size,
            save_command.clone(),
        ))
        .with_child(size_row(
            "URL label size",
            CustomPalette::url_label_size,
            save_command.clone(),
        ));

    let custom_palette_section = Either::new(
        |data: &UIState, _env| data.ui_settings.visual_settings.theme == ConfiguredTheme::Custom,
        custom_palette_section,
        Flex::column(),
    );

    return Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(theme_radio_row)
        .with_spacer(12.0)
        .with_child(custom_palette_section)
        .scroll()
        .vertical();
}

fn section_label(text: &str) -> impl Widget<UIState> {
    Label::new(text.to_string())
        .with_text_size(12.0)
        .padding((0.0, 4.0))
}

fn custom_palette_field<T: Data>(
    field_lens: impl Lens<CustomPalette, T> + 'static,
) -> impl Lens<UIState, T> {
    UIState::ui_settings
        .then(UISettings::visual_settings)
        .then(UIVisualSettings::custom_palette)
        .then(field_lens)
}

fn color_row(
    label: &str,
    field_lens: impl Lens<CustomPalette, String> + 'static,
    save_command: Command,
) -> impl Widget<UIState> {
    text_row(label, field_lens, save_command)
}

fn text_row(
    label: &str,
    field_lens: impl Lens<CustomPalette, String> + 'static,
    save_command: Command,
) -> impl Widget<UIState> {
    let input = ControllerHost::new(
        TextBox::new(),
        rules_view::SubmitCommandOnDataChange {
            command: save_command,
        },
    )
    .lens(custom_palette_field(field_lens));

    Flex::row()
        .with_child(Label::new(label.to_string()).with_text_size(12.0))
        .with_flex_spacer(1.0)
        .with_child(input.fix_width(160.0))
        .padding((0.0, 0.0, 0.0, 4.0))
}

fn size_row(
    label: &str,
    field_lens: impl Lens<CustomPalette, f64> + 'static,
    save_command: Command,
) -> impl Widget<UIState> {
    let input = ControllerHost::new(
        TextBox::new(),
        rules_view::SubmitCommandOnDataChange {
            command: save_command,
        },
    )
    .lens(custom_palette_field(field_lens).then(FloatAsString));

    Flex::row()
        .with_child(Label::new(label.to_string()).with_text_size(12.0))
        .with_flex_spacer(1.0)
        .with_child(input.fix_width(160.0))
        .padding((0.0, 0.0, 0.0, 4.0))
}

/// Adapts a `f64` field to a `String` text box, keeping the previous value
/// when the box contains text that doesn't parse as a number.
struct FloatAsString;

impl Lens<f64, String> for FloatAsString {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &f64, f: F) -> V {
        f(&data.to_string())
    }

    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut f64, f: F) -> V {
        let mut text = data.to_string();
        let result = f(&mut text);
        if let Ok(parsed) = text.parse::<f64>() {
            *data = parsed;
        }
        result
    }
}
