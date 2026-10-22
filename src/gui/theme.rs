use slint::ComponentHandle;

use crate::utils::ConfiguredTheme;

pub type ColorScheme = slint::language::ColorScheme;

// the actual light/dark colors live in theme.slint as bindings on the effective color scheme, so
// OS appearance changes are picked up there without any Rust involvement. all Rust does is pass
// in the user's explicit Theme choice (Auto maps to Unknown, meaning "follow the OS").
pub fn configured_color_scheme(theme: ConfiguredTheme) -> ColorScheme {
    match theme {
        ConfiguredTheme::Auto => ColorScheme::Unknown,
        ConfiguredTheme::Light => ColorScheme::Light,
        ConfiguredTheme::Dark => ColorScheme::Dark,
    }
}

// generic over the window type - MainWindow/SettingsWindow/AboutWindow each get their own
// independent AppPalette global instance (separate compile units), but all implement
// slint::Global for it the same way, so one function covers all three
pub fn apply_to_window<'a, T>(scheme: ColorScheme, window: &'a T)
where
    T: ComponentHandle,
    crate::gui::generated::AppPalette<'a>: slint::Global<'a, T>,
{
    use crate::gui::generated::AppPalette as SlintPalette;
    window.global::<SlintPalette>().set_scheme_override(scheme);
}
