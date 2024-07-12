use dark_light::Mode;
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
    let active_ui_theme = get_active_ui_theme(ui_state);
    setup_theme(env, active_ui_theme);
}

fn get_active_ui_theme(ui_state: &UIState) -> UITheme {
    let system_ui_theme = detect_system_theme();
    return match ui_state.ui_settings.visual_settings.theme {
        ConfiguredTheme::Auto => system_ui_theme,
        ConfiguredTheme::Light => UITheme::Light,
        ConfiguredTheme::Dark => UITheme::Dark,
    };
}

fn detect_system_theme() -> UITheme {
    let system_dark_light_mode: Mode = dark_light::detect();
    return match system_dark_light_mode {
        Mode::Dark => UITheme::Dark,
        Mode::Light => UITheme::Light,
        Mode::Default => UITheme::Dark,
    };
}

pub fn setup_theme(env: &mut Env, ui_theme: UITheme) {
    let theme = get_theme(ui_theme);
    theme.set_env_to_theme(env);
}

fn get_theme(ui_theme: UITheme) -> Theme {
    let dark_theme = Theme {
        druid_builtin: DruidBuiltinTheme {
            window_background_color: Color::rgb8(0x29, 0x29, 0x29),
            text_color: Color::rgb8(0xf0, 0xf0, 0xea),
            disabled_text_color: Color::rgb8(0xa0, 0xa0, 0x9a),
            placeholder_color: Color::rgb8(0x80, 0x80, 0x80),
            primary_light: Color::rgb8(0x5c, 0xc4, 0xff),
            primary_dark: Color::rgb8(0x00, 0x8d, 0xdd),
            background_light: Color::rgb8(0x3a, 0x3a, 0x3a),
            background_dark: Color::rgb8(0x31, 0x31, 0x31),
            foreground_light: Color::rgb8(0xf9, 0xf9, 0xf9),
            foreground_dark: Color::rgb8(0xbf, 0xbf, 0xbf),
            disabled_foreground_light: Color::rgb8(0x89, 0x89, 0x89),
            disabled_foreground_dark: Color::rgb8(0x6f, 0x6f, 0x6f),
            button_dark: Color::BLACK,
            button_light: Color::rgb8(0x21, 0x21, 0x21),
            disabled_button_dark: Color::grey8(0x28),
            disabled_button_light: Color::grey8(0x38),
            border_dark: Color::rgb8(0x3a, 0x3a, 0x3a),
            border_light: Color::rgb8(0xa1, 0xa1, 0xa1),
            selected_text_background_color: Color::rgb8(0x43, 0x70, 0xA8),
            selected_text_inactive_background_color: Color::grey8(0x74),
            selection_text_color: Color::rgb8(0x00, 0x00, 0x00),
            cursor_color: Color::WHITE,
            scrollbar_color: Color::rgb8(0xff, 0xff, 0xff),
            scrollbar_border_color: Color::rgb8(0x77, 0x77, 0x77),
        },
        general: GeneralTheme {
            window_background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        main: MainWindowTheme {
            window_background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
            profile_label_color: Color::rgb8(190, 190, 190),
            browser_label_color: Color::rgb8(255, 255, 255),
            hotkey_background_color: Color::rgba(0.15, 0.15, 0.15, 1.0),
            hotkey_border_color: Color::rgba(0.4, 0.4, 0.4, 0.9),
            hotkey_text_color: Color::rgb8(128, 128, 128),
            options_button_text_color: Color::rgb8(128, 128, 128),
        },
        settings: SettingsWindowTheme {
            active_tab_background_color: Color::rgb8(25, 90, 194),
            active_tab_text_color: Color::rgb8(255, 255, 255),
            inactive_tab_text_color: Color::rgb8(255, 255, 255),
            rule_background_color: Color::rgba(0.1, 0.1, 0.1, 0.9),
            rule_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        about: AboutWindowTheme {
            window_background_color: Color::rgb8(27, 32, 32),
        },
    };

    let light_theme = Theme {
        druid_builtin: DruidBuiltinTheme {
            window_background_color: Color::rgb(0.85, 0.85, 0.85),
            text_color: Color::rgb8(10, 10, 10),
            disabled_text_color: Color::rgb8(0xa0, 0xa0, 0x9a),
            placeholder_color: Color::rgb8(0x80, 0x80, 0x80),
            primary_light: Color::rgb8(0x5c, 0xc4, 0xff),
            primary_dark: Color::rgb8(0x00, 0x8d, 0xdd),
            background_light: Color::rgb8(220, 220, 220),
            background_dark: Color::rgb8(200, 200, 200),
            foreground_light: Color::rgb8(0xf9, 0xf9, 0xf9),
            foreground_dark: Color::rgb8(0xbf, 0xbf, 0xbf),
            disabled_foreground_light: Color::rgb8(0x89, 0x89, 0x89),
            disabled_foreground_dark: Color::rgb8(0x6f, 0x6f, 0x6f),
            button_dark: Color::rgb8(120, 120, 120),
            button_light: Color::rgb8(150, 150, 150),
            disabled_button_dark: Color::grey8(0x28),
            disabled_button_light: Color::grey8(0x38),
            border_dark: Color::rgb8(0x3a, 0x3a, 0x3a),
            border_light: Color::rgb8(0xa1, 0xa1, 0xa1),
            selected_text_background_color: Color::rgb8(0x43, 0x70, 0xA8),
            selected_text_inactive_background_color: Color::grey8(0x74),
            selection_text_color: Color::rgb8(0x00, 0x00, 0x00),
            cursor_color: Color::BLACK,
            scrollbar_color: Color::rgb8(0xff, 0xff, 0xff),
            scrollbar_border_color: Color::rgb8(0x77, 0x77, 0x77),
        },
        general: GeneralTheme {
            window_background_color: Color::rgba(0.85, 0.85, 0.85, 0.9),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        main: MainWindowTheme {
            window_background_color: Color::rgba8(215, 215, 215, 230),
            window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
            profile_label_color: Color::rgb8(30, 30, 30),
            browser_label_color: Color::rgb8(0, 0, 0),
            hotkey_background_color: Color::rgb8(215, 215, 215),
            hotkey_border_color: Color::rgba(0.4, 0.4, 0.4, 0.9),
            hotkey_text_color: Color::rgb8(128, 128, 128),
            options_button_text_color: Color::rgb8(128, 128, 128),
        },
        settings: SettingsWindowTheme {
            active_tab_background_color: Color::rgb8(25, 90, 194),
            active_tab_text_color: Color::rgb8(255, 255, 255),
            inactive_tab_text_color: Color::rgb8(0, 0, 0),
            rule_background_color: Color::rgba(0.8, 0.8, 0.8, 0.9),
            rule_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
        },
        about: AboutWindowTheme {
            window_background_color: Color::rgb8(236, 236, 236),
        },
    };

    let theme = match ui_theme {
        UITheme::Light => light_theme,
        UITheme::Dark => dark_theme,
    };

    return theme;
}

struct Theme {
    druid_builtin: DruidBuiltinTheme,
    general: GeneralTheme,
    main: MainWindowTheme,
    settings: SettingsWindowTheme,
    about: AboutWindowTheme,
}

impl Theme {
    fn set_env_to_theme(&self, env: &mut Env) {
        self.druid_builtin.set_env_to_theme(env);
        self.general.set_env_to_theme(env);
        self.main.set_env_to_theme(env);
        self.settings.set_env_to_theme(env);
        self.about.set_env_to_theme(env);
    }
}

pub(crate) struct GeneralTheme {
    window_background_color: Color,
    window_border_color: Color,
}

impl GeneralTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.general.window_background_color");
    pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.general.window_border_color");

    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(Self::ENV_WINDOW_BACKGROUND_COLOR, self.window_background_color);
        env.set(Self::ENV_WINDOW_BORDER_COLOR, self.window_border_color);
    }
}

pub(crate) struct MainWindowTheme {
    window_background_color: Color,
    window_border_color: Color,
    profile_label_color: Color,
    browser_label_color: Color,
    hotkey_background_color: Color,
    hotkey_border_color: Color,
    hotkey_text_color: Color,
    options_button_text_color: Color,
}

impl MainWindowTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_background_color");

    pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_border_color");

    pub const ENV_PROFILE_LABEL_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.profile_label_color");

    pub const ENV_BROWSER_LABEL_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.browser_label_color");

    pub const ENV_HOTKEY_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_background_color");

    pub const ENV_HOTKEY_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_border_color");

    pub const ENV_HOTKEY_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_text_color");

    pub const ENV_OPTIONS_BUTTON_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.options_button_text_color");

    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(Self::ENV_WINDOW_BACKGROUND_COLOR, self.window_background_color);
        env.set(Self::ENV_WINDOW_BORDER_COLOR, self.window_border_color);
        env.set(Self::ENV_PROFILE_LABEL_COLOR, self.profile_label_color);
        env.set(Self::ENV_BROWSER_LABEL_COLOR, self.browser_label_color);
        env.set(Self::ENV_HOTKEY_BACKGROUND_COLOR, self.hotkey_background_color);
        env.set(Self::ENV_HOTKEY_BORDER_COLOR, self.hotkey_border_color);
        env.set(Self::ENV_HOTKEY_TEXT_COLOR, self.hotkey_text_color);
        env.set(
            Self::ENV_OPTIONS_BUTTON_TEXT_COLOR,
            self.options_button_text_color,
        );
    }
}

pub(crate) struct SettingsWindowTheme {
    active_tab_background_color: Color,
    active_tab_text_color: Color,
    inactive_tab_text_color: Color,
    rule_background_color: Color,
    rule_border_color: Color,
}

impl SettingsWindowTheme {
    pub const ENV_ACTIVE_TAB_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.settings.active_tab_background_color");

    pub const ENV_ACTIVE_TAB_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.settings.active_tab_text_color");

    pub const ENV_INACTIVE_TAB_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.settings.inactive_tab_text_color");

    pub const ENV_RULE_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.settings.rule_background_color");

    pub const ENV_RULE_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.settings.rule_border_color");

    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(
            Self::ENV_ACTIVE_TAB_BACKGROUND_COLOR,
            self.active_tab_background_color,
        );

        env.set(Self::ENV_ACTIVE_TAB_TEXT_COLOR, self.active_tab_text_color);
        env.set(Self::ENV_INACTIVE_TAB_TEXT_COLOR, self.inactive_tab_text_color);

        env.set(Self::ENV_RULE_BACKGROUND_COLOR, self.rule_background_color);
        env.set(Self::ENV_RULE_BORDER_COLOR, self.rule_border_color);
    }
}

pub(crate) struct AboutWindowTheme {
    window_background_color: Color,
}

impl AboutWindowTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.about.window_background_color");

    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(Self::ENV_WINDOW_BACKGROUND_COLOR, self.window_background_color);
    }
}

pub(crate) struct DruidBuiltinTheme {
    window_background_color: Color,
    text_color: Color,
    disabled_text_color: Color,
    placeholder_color: Color,
    primary_light: Color,
    primary_dark: Color,
    background_light: Color,
    background_dark: Color,
    foreground_light: Color,
    foreground_dark: Color,
    disabled_foreground_light: Color,
    disabled_foreground_dark: Color,
    button_dark: Color,
    button_light: Color,
    disabled_button_dark: Color,
    disabled_button_light: Color,
    border_dark: Color,
    border_light: Color,
    selected_text_background_color: Color,
    selected_text_inactive_background_color: Color,
    selection_text_color: Color,
    cursor_color: Color,
    scrollbar_color: Color,
    scrollbar_border_color: Color,
}

impl DruidBuiltinTheme {
    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(
            druid::theme::WINDOW_BACKGROUND_COLOR,
            self.window_background_color,
        );
        env.set(druid::theme::TEXT_COLOR, self.text_color);
        env.set(druid::theme::DISABLED_TEXT_COLOR, self.disabled_text_color);
        env.set(druid::theme::PLACEHOLDER_COLOR, self.placeholder_color);
        env.set(druid::theme::PRIMARY_LIGHT, self.primary_light);
        env.set(druid::theme::PRIMARY_DARK, self.primary_dark);
        env.set(druid::theme::BACKGROUND_LIGHT, self.background_light);
        env.set(druid::theme::BACKGROUND_DARK, self.background_dark);
        env.set(druid::theme::FOREGROUND_LIGHT, self.foreground_light);
        env.set(druid::theme::FOREGROUND_DARK, self.foreground_dark);
        env.set(
            druid::theme::DISABLED_FOREGROUND_LIGHT,
            self.disabled_foreground_light,
        );
        env.set(
            druid::theme::DISABLED_FOREGROUND_DARK,
            self.disabled_foreground_dark,
        );
        env.set(druid::theme::BUTTON_DARK, self.button_dark);
        env.set(druid::theme::BUTTON_LIGHT, self.button_light);
        env.set(druid::theme::DISABLED_BUTTON_DARK, self.disabled_button_dark);
        env.set(druid::theme::DISABLED_BUTTON_LIGHT, self.disabled_button_light);
        env.set(druid::theme::BORDER_DARK, self.border_dark);
        env.set(druid::theme::BORDER_LIGHT, self.border_light);
        env.set(
            druid::theme::SELECTED_TEXT_BACKGROUND_COLOR,
            self.selected_text_background_color,
        );
        env.set(
            druid::theme::SELECTED_TEXT_INACTIVE_BACKGROUND_COLOR,
            self.selected_text_inactive_background_color,
        );
        env.set(druid::theme::SELECTION_TEXT_COLOR, self.selection_text_color);
        env.set(druid::theme::CURSOR_COLOR, self.cursor_color);
        env.set(druid::theme::SCROLLBAR_COLOR, self.scrollbar_color);
        env.set(druid::theme::SCROLLBAR_BORDER_COLOR, self.scrollbar_border_color);
    }
}

struct Palette {}

//.adding(PROGRESS_BAR_RADIUS, 4.)
//.adding(BUTTON_BORDER_RADIUS, 4.)
//.adding(BUTTON_BORDER_WIDTH, 2.)
//.adding(TEXT_SIZE_NORMAL, 15.0)
//.adding(TEXT_SIZE_LARGE, 24.0)
//.adding(BASIC_WIDGET_HEIGHT, 18.0)
//.adding(WIDE_WIDGET_WIDTH, 100.)
//.adding(BORDERED_WIDGET_HEIGHT, 24.0)
//.adding(TEXTBOX_BORDER_RADIUS, 2.)
//.adding(TEXTBOX_BORDER_WIDTH, 1.)
//.adding(TEXTBOX_INSETS, Insets::new(4.0, 4.0, 4.0, 4.0))
//.adding(SCROLLBAR_MAX_OPACITY, 0.7)
//.adding(SCROLLBAR_FADE_DELAY, 1500u64)
//.adding(SCROLLBAR_WIDTH, 8.)
//.adding(SCROLLBAR_PAD, 2.)
//.adding(SCROLLBAR_MIN_SIZE, 45.)
//.adding(SCROLLBAR_RADIUS, 5.)
//.adding(SCROLLBAR_EDGE_WIDTH, 1.)
//.adding(WIDGET_PADDING_VERTICAL, 10.0)
//.adding(WIDGET_PADDING_HORIZONTAL, 8.0)
//.adding(WIDGET_CONTROL_COMPONENT_PADDING, 4.0)
//.adding(UI_FONT, FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(15.0))
//.adding(UI_FONT_BOLD, FontDescriptor::new(FontFamily::SYSTEM_UI).with_weight(FontWeight::BOLD).with_size(15.0))
//.adding(UI_FONT_ITALIC, FontDescriptor::new(FontFamily::SYSTEM_UI).with_style(FontStyle::Italic).with_size(15.0))
