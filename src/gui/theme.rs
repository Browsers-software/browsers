use slint::{Color, ComponentHandle};

use crate::gui::app_state::UIVisualSettings;
use crate::utils::ConfiguredTheme;

pub type ColorScheme = slint::language::ColorScheme;

// we use Slint's own Palette.color-scheme for "Auto" theme now instead of the separate dark-light
// crate we had before (it didn't reflect the actual macOS setting for some users) - this is also
// what native Switch/LineEdit widgets already track, so our colors and theirs can't disagree.
//
// one wrinkle: before the main window's backend exists, color-scheme just reports Unknown instead
// of the real setting, which shows up as a light-then-dark flash on launch.
// so for that narrow startup gap only, we ask macOS directly (macos_native::is_dark_mode) - once
// Slint starts reporting a real value it takes over for good, so there's still just one source of
// truth once we're actually running
pub fn detect_system_is_dark(window: &crate::gui::generated::MainWindow) -> bool {
    use crate::gui::generated::Palette as StdWidgetPalette;
    match window.global::<StdWidgetPalette>().get_color_scheme() {
        ColorScheme::Dark => true,
        ColorScheme::Light => false,
        _ => {
            #[cfg(target_os = "macos")]
            {
                crate::macos::macos_native::is_dark_mode()
            }
            #[cfg(not(target_os = "macos"))]
            {
                false
            }
        }
    }
}

pub struct Palette {
    pub background_color: Color,
    pub label_color: Color,
    pub secondary_label_color: Color,
    pub muted_label_color: Color,
    pub stroke_color: Color,
    pub secondary_stroke_color: Color,
    pub highlight_background_color: Color,
    pub secondary_background_color: Color,
    pub subtle_background_color: Color,
    pub accent_color: Color,
    pub on_accent_color: Color,
    pub opaque_background_color: Color,
    pub text_color: Color,
    pub background_light: Color,
    pub background_dark: Color,
    pub button_dark: Color,
    pub button_light: Color,
    pub cursor_color: Color,
    pub about_background_color: Color,
    pub font_family: String,
    pub browser_label_size: f32,
    pub profile_label_size: f32,
    pub url_label_size: f32,
    pub label_color_hover: Color,
    pub secondary_label_color_hover: Color,
    pub hotkey_background_color_hover: Color,
    pub hotkey_text_color_hover: Color,
}

fn argb(a: f32, r: f32, g: f32, b: f32) -> Color {
    Color::from_argb_u8(
        (a * 255.0).round() as u8,
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

impl Palette {
    fn dark() -> Self {
        Palette {
            background_color: argb(0.9, 0.15, 0.15, 0.15),
            label_color: Color::from_rgb_u8(255, 255, 255),
            secondary_label_color: Color::from_rgb_u8(190, 190, 190),
            muted_label_color: Color::from_rgb_u8(128, 128, 128),
            stroke_color: argb(0.1, 1.0, 1.0, 1.0),
            secondary_stroke_color: argb(0.9, 0.4, 0.4, 0.4),
            highlight_background_color: argb(0.25, 1.0, 1.0, 1.0),
            secondary_background_color: Color::from_rgb_u8(38, 38, 38),
            subtle_background_color: argb(0.9, 0.1, 0.1, 0.1),
            accent_color: Color::from_rgb_u8(25, 90, 194),
            on_accent_color: Color::from_rgb_u8(255, 255, 255),
            opaque_background_color: Color::from_rgb_u8(0x29, 0x29, 0x29),
            text_color: Color::from_rgb_u8(0xf0, 0xf0, 0xea),
            background_light: Color::from_rgb_u8(0x3a, 0x3a, 0x3a),
            background_dark: Color::from_rgb_u8(0x31, 0x31, 0x31),
            button_dark: Color::from_rgb_u8(0, 0, 0),
            button_light: Color::from_rgb_u8(0x21, 0x21, 0x21),
            cursor_color: Color::from_rgb_u8(255, 255, 255),
            about_background_color: Color::from_rgb_u8(27, 32, 32),
            font_family: String::new(),
            browser_label_size: 12.0,
            profile_label_size: 11.0,
            url_label_size: 12.0,
            label_color_hover: Color::from_rgb_u8(255, 255, 255),
            secondary_label_color_hover: Color::from_rgb_u8(255, 255, 255),
            hotkey_background_color_hover: Color::from_rgb_u8(38, 38, 38),
            hotkey_text_color_hover: Color::from_rgb_u8(255, 255, 255),
        }
    }

    fn light() -> Self {
        Palette {
            background_color: argb(0.9, 0.85, 0.85, 0.85),
            label_color: Color::from_rgb_u8(0, 0, 0),
            secondary_label_color: Color::from_rgb_u8(30, 30, 30),
            muted_label_color: Color::from_rgb_u8(128, 128, 128),
            stroke_color: argb(0.9, 0.7, 0.7, 0.7),
            secondary_stroke_color: argb(0.9, 0.4, 0.4, 0.4),
            highlight_background_color: argb(0.25, 1.0, 1.0, 1.0),
            secondary_background_color: Color::from_rgb_u8(215, 215, 215),
            subtle_background_color: argb(0.9, 0.8, 0.8, 0.8),
            accent_color: Color::from_rgb_u8(25, 90, 194),
            on_accent_color: Color::from_rgb_u8(255, 255, 255),
            opaque_background_color: Color::from_argb_u8(255, 217, 217, 217),
            text_color: Color::from_rgb_u8(10, 10, 10),
            background_light: Color::from_rgb_u8(220, 220, 220),
            background_dark: Color::from_rgb_u8(200, 200, 200),
            button_dark: Color::from_rgb_u8(120, 120, 120),
            button_light: Color::from_rgb_u8(150, 150, 150),
            cursor_color: Color::from_rgb_u8(0, 0, 0),
            about_background_color: Color::from_rgb_u8(236, 236, 236),
            font_family: String::new(),
            browser_label_size: 12.0,
            profile_label_size: 11.0,
            url_label_size: 12.0,
            label_color_hover: Color::from_rgb_u8(0, 0, 0),
            secondary_label_color_hover: Color::from_rgb_u8(0, 0, 0),
            hotkey_background_color_hover: Color::from_rgb_u8(215, 215, 215),
            hotkey_text_color_hover: Color::from_rgb_u8(0, 0, 0),
        }
    }
}

pub fn resolve_palette(visual_settings: &UIVisualSettings, system_is_dark: bool) -> Palette {
    match visual_settings.theme {
        ConfiguredTheme::Auto => {
            if system_is_dark {
                Palette::dark()
            } else {
                Palette::light()
            }
        }
        ConfiguredTheme::Light => Palette::light(),
        ConfiguredTheme::Dark => Palette::dark(),
    }
}

// pushes every Palette field onto a generated Slint Palette global (one per window - each .slint
// document compiles its own copy of theme.slint's import)
macro_rules! apply_palette {
    ($globals:expr, $p:expr) => {{
        let g = $globals;
        let p: &Palette = $p;
        g.set_background_color(p.background_color);
        g.set_label_color(p.label_color);
        g.set_secondary_label_color(p.secondary_label_color);
        g.set_muted_label_color(p.muted_label_color);
        g.set_stroke_color(p.stroke_color);
        g.set_secondary_stroke_color(p.secondary_stroke_color);
        g.set_highlight_background_color(p.highlight_background_color);
        g.set_secondary_background_color(p.secondary_background_color);
        g.set_subtle_background_color(p.subtle_background_color);
        g.set_accent_color(p.accent_color);
        g.set_on_accent_color(p.on_accent_color);
        g.set_opaque_background_color(p.opaque_background_color);
        g.set_text_color(p.text_color);
        g.set_background_light(p.background_light);
        g.set_background_dark(p.background_dark);
        g.set_button_dark(p.button_dark);
        g.set_button_light(p.button_light);
        g.set_cursor_color(p.cursor_color);
        g.set_about_background_color(p.about_background_color);
        g.set_font_family(p.font_family.clone().into());
        g.set_browser_label_size(p.browser_label_size);
        g.set_profile_label_size(p.profile_label_size);
        g.set_url_label_size(p.url_label_size);
        g.set_label_color_hover(p.label_color_hover);
        g.set_secondary_label_color_hover(p.secondary_label_color_hover);
        g.set_hotkey_background_color_hover(p.hotkey_background_color_hover);
        g.set_hotkey_text_color_hover(p.hotkey_text_color_hover);
    }};
}

pub fn apply_to_main_window(palette: &Palette, window: &crate::gui::generated::MainWindow) {
    use crate::gui::generated::AppPalette as SlintPalette;
    apply_palette!(window.global::<SlintPalette>(), palette);
}

pub fn apply_to_settings_window(palette: &Palette, window: &crate::gui::generated::SettingsWindow) {
    use crate::gui::generated::AppPalette as SlintPalette;
    apply_palette!(window.global::<SlintPalette>(), palette);
}

pub fn apply_to_about_window(palette: &Palette, window: &crate::gui::generated::AboutWindow) {
    use crate::gui::generated::AppPalette as SlintPalette;
    apply_palette!(window.global::<SlintPalette>(), palette);
}
