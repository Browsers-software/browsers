use druid::{Color, Data, Env, Key};
use serde::{Deserialize, Serialize};

use crate::gui::ui::UIState;
use crate::utils::ConfiguredTheme;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Data)]
pub enum UITheme {
    Light,
    Dark,
}

pub fn initialize_theme(env: &mut Env, ui_state: &UIState) {
    let ui_theme: UITheme = match ui_state.ui_settings.visual_settings.theme {
        ConfiguredTheme::Auto => UITheme::Dark,
        ConfiguredTheme::Light => UITheme::Light,
        ConfiguredTheme::Dark => UITheme::Dark,
    };

    setup_theme(env, ui_theme);
}

pub fn setup_theme(env: &mut Env, ui_theme: UITheme) {
    let dark_theme = Theme {
        general: GeneralTheme {
            window_background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        main: MainWindowTheme {
            window_background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        about: AboutWindowTheme {
            window_background_color: Color::rgb8(27, 32, 32),
        },
    };

    let light_theme = Theme {
        general: GeneralTheme {
            window_background_color: Color::rgba(0.85, 0.85, 0.85, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        main: MainWindowTheme {
            window_background_color: Color::rgba8(215, 215, 215, 230),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        about: AboutWindowTheme {
            window_background_color: Color::rgb8(236, 236, 236),
        },
    };

    let theme = match ui_theme {
        UITheme::Light => light_theme,
        UITheme::Dark => dark_theme,
    };

    set_env_to_theme(env, theme);
}

fn set_env_to_theme(env: &mut Env, theme: Theme) {
    env.set(
        GeneralTheme::ENV_WINDOW_BACKGROUND_COLOR,
        theme.general.window_background_color,
    );
    env.set(
        GeneralTheme::ENV_WINDOW_BORDER_COLOR,
        theme.general.window_border_color,
    );

    env.set(
        MainWindowTheme::ENV_WINDOW_BACKGROUND_COLOR,
        theme.main.window_background_color,
    );
    env.set(
        MainWindowTheme::ENV_WINDOW_BORDER_COLOR,
        theme.main.window_border_color,
    );

    env.set(
        AboutWindowTheme::ENV_WINDOW_BACKGROUND_COLOR,
        theme.about.window_background_color,
    );
}

struct Theme {
    general: GeneralTheme,
    main: MainWindowTheme,
    about: AboutWindowTheme,
}

struct GeneralTheme {
    window_background_color: Color,
    window_border_color: Color,
}

impl GeneralTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.general.window_background_color");
    pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.general.window_border_color");
}

pub(crate) struct MainWindowTheme {
    window_background_color: Color,
    window_border_color: Color,
}

impl MainWindowTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_background_color");

    pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_border_color");
}

pub(crate) struct AboutWindowTheme {
    window_background_color: Color,
}

impl AboutWindowTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.about.window_background_color");
}

struct Palette {}

/*
env.adding(WINDOW_BACKGROUND_COLOR, Color::rgb8(0x29, 0x29, 0x29))
       .adding(TEXT_COLOR, Color::rgb8(0xf0, 0xf0, 0xea))
       .adding(DISABLED_TEXT_COLOR, Color::rgb8(0xa0, 0xa0, 0x9a))
       .adding(PLACEHOLDER_COLOR, Color::rgb8(0x80, 0x80, 0x80))
       .adding(PRIMARY_LIGHT, Color::rgb8(0x5c, 0xc4, 0xff))
       .adding(PRIMARY_DARK, Color::rgb8(0x00, 0x8d, 0xdd))
       .adding(PROGRESS_BAR_RADIUS, 4.)
       .adding(BACKGROUND_LIGHT, Color::rgb8(0x3a, 0x3a, 0x3a))
       .adding(BACKGROUND_DARK, Color::rgb8(0x31, 0x31, 0x31))
       .adding(FOREGROUND_LIGHT, Color::rgb8(0xf9, 0xf9, 0xf9))
       .adding(FOREGROUND_DARK, Color::rgb8(0xbf, 0xbf, 0xbf))
       .adding(DISABLED_FOREGROUND_LIGHT, Color::rgb8(0x89, 0x89, 0x89))
       .adding(DISABLED_FOREGROUND_DARK, Color::rgb8(0x6f, 0x6f, 0x6f))
       .adding(BUTTON_DARK, Color::BLACK)
       .adding(BUTTON_LIGHT, Color::rgb8(0x21, 0x21, 0x21))
       .adding(DISABLED_BUTTON_DARK, Color::grey8(0x28))
       .adding(DISABLED_BUTTON_LIGHT, Color::grey8(0x38))
       .adding(BUTTON_BORDER_RADIUS, 4.)
       .adding(BUTTON_BORDER_WIDTH, 2.)
       .adding(BORDER_DARK, Color::rgb8(0x3a, 0x3a, 0x3a))
       .adding(BORDER_LIGHT, Color::rgb8(0xa1, 0xa1, 0xa1))
       .adding(
           SELECTED_TEXT_BACKGROUND_COLOR,
           Color::rgb8(0x43, 0x70, 0xA8),
       )
       .adding(SELECTED_TEXT_INACTIVE_BACKGROUND_COLOR, Color::grey8(0x74))
       .adding(SELECTION_TEXT_COLOR, Color::rgb8(0x00, 0x00, 0x00))
       .adding(CURSOR_COLOR, Color::WHITE)
       .adding(TEXT_SIZE_NORMAL, 15.0)
       .adding(TEXT_SIZE_LARGE, 24.0)
       .adding(BASIC_WIDGET_HEIGHT, 18.0)
       .adding(WIDE_WIDGET_WIDTH, 100.)
       .adding(BORDERED_WIDGET_HEIGHT, 24.0)
       .adding(TEXTBOX_BORDER_RADIUS, 2.)
       .adding(TEXTBOX_BORDER_WIDTH, 1.)
       .adding(TEXTBOX_INSETS, Insets::new(4.0, 4.0, 4.0, 4.0))
       .adding(SCROLLBAR_COLOR, Color::rgb8(0xff, 0xff, 0xff))
       .adding(SCROLLBAR_BORDER_COLOR, Color::rgb8(0x77, 0x77, 0x77))
       .adding(SCROLLBAR_MAX_OPACITY, 0.7)
       .adding(SCROLLBAR_FADE_DELAY, 1500u64)
       .adding(SCROLLBAR_WIDTH, 8.)
       .adding(SCROLLBAR_PAD, 2.)
       .adding(SCROLLBAR_MIN_SIZE, 45.)
       .adding(SCROLLBAR_RADIUS, 5.)
       .adding(SCROLLBAR_EDGE_WIDTH, 1.)
       .adding(WIDGET_PADDING_VERTICAL, 10.0)
       .adding(WIDGET_PADDING_HORIZONTAL, 8.0)
       .adding(WIDGET_CONTROL_COMPONENT_PADDING, 4.0)
       .adding(
           UI_FONT,
           FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(15.0),
       )
       .adding(
           UI_FONT_BOLD,
           FontDescriptor::new(FontFamily::SYSTEM_UI)
               .with_weight(FontWeight::BOLD)
               .with_size(15.0),
       )
       .adding(
           UI_FONT_ITALIC,
           FontDescriptor::new(FontFamily::SYSTEM_UI)
               .with_style(FontStyle::Italic)
               .with_size(15.0),
       )
 **/
