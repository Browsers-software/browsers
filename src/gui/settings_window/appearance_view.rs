use druid::widget::{ControllerHost, CrossAxisAlignment, Either, Flex, Label, RadioGroup, TextBox};
use druid::{
    Command, Data, FontDescriptor, FontFamily, FontWeight, Lens, LensExt, Widget, WidgetExt,
};

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
        .with_child(section_label("Window"))
        .with_child(color_row(
            "Background",
            CustomPalette::background_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Border",
            CustomPalette::stroke_color,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("Browser & profile rows"))
        .with_child(color_row(
            "Browser label",
            CustomPalette::label_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Browser label (hover)",
            CustomPalette::label_color_hover,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Profile label",
            CustomPalette::secondary_label_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Profile label (hover)",
            CustomPalette::secondary_label_color_hover,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Row background (hover)",
            CustomPalette::highlight_background_color,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("URL & hotkeys"))
        .with_child(color_row(
            "URL / hotkey / options text",
            CustomPalette::muted_label_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey background",
            CustomPalette::secondary_background_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey background (hover)",
            CustomPalette::hotkey_background_color_hover,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey border",
            CustomPalette::secondary_stroke_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Hotkey text (hover)",
            CustomPalette::hotkey_text_color_hover,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("Settings tabs & rules"))
        .with_child(color_row(
            "Active tab background",
            CustomPalette::accent_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Active tab text",
            CustomPalette::on_accent_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Rule background",
            CustomPalette::subtle_background_color,
            save_command.clone(),
        ))
        .with_spacer(8.0)
        .with_child(section_label("About window"))
        .with_child(color_row(
            "Background",
            CustomPalette::about_background_color,
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
        ))
        .with_spacer(8.0)
        .with_child(section_label("Native widgets (rarely visible)"))
        .with_child(color_row(
            "Background (opaque)",
            CustomPalette::opaque_background_color,
            save_command.clone(),
        ))
        .with_child(color_row(
            "Text",
            CustomPalette::text_color,
            save_command.clone(),
        ))
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
        .with_child(color_row(
            "Cursor",
            CustomPalette::cursor_color,
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
    let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(12.0);

    Label::new(text.to_string())
        .with_font(font)
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
