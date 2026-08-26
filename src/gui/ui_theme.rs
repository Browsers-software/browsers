use crate::gui::ui::{UIState, UIVisualSettings};
use crate::utils::{ConfiguredTheme, CustomPalette};
use dark_light::Mode;
use druid::{Color, Data, Env, FontDescriptor, FontFamily, FontStyle, FontWeight, Insets, Key};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Data)]
pub enum UITheme {
    Light,
    Dark,
}

pub fn initialize_theme(env: &mut Env, ui_state: &UIState) {
    setup_theme(env, &ui_state.ui_settings.visual_settings);
}

fn detect_system_theme() -> UITheme {
    match dark_light::detect() {
        Ok(Mode::Dark) => UITheme::Dark,
        Ok(Mode::Light) => UITheme::Light,
        Ok(Mode::Unspecified) => UITheme::Dark,
        Err(error) => {
            warn!("{}", error);
            UITheme::Dark
        }
    }
}

pub fn setup_theme(env: &mut Env, visual_settings: &UIVisualSettings) {
    let theme = get_theme(visual_settings);
    theme.set_env_to_theme(env);
}

fn get_theme(visual_settings: &UIVisualSettings) -> Theme {
    let palette = resolve_palette(visual_settings);
    build_theme(&palette)
}

fn resolve_palette(visual_settings: &UIVisualSettings) -> Palette {
    match visual_settings.theme {
        ConfiguredTheme::Auto => match detect_system_theme() {
            UITheme::Dark => Palette::dark(),
            UITheme::Light => Palette::light(),
        },
        ConfiguredTheme::Light => Palette::light(),
        ConfiguredTheme::Dark => Palette::dark(),
        ConfiguredTheme::Custom => Palette::from_custom(&visual_settings.custom_palette),
    }
}

fn build_theme(palette: &Palette) -> Theme {
    Theme {
        druid_builtin: DruidBuiltinTheme {
            window_background_color: palette.opaque_background_color,
            text_color: palette.text_color,
            disabled_text_color: Color::rgb8(0xa0, 0xa0, 0x9a),
            placeholder_color: Color::rgb8(0x80, 0x80, 0x80),
            primary_light: Color::rgb8(0x5c, 0xc4, 0xff),
            primary_dark: Color::rgb8(0x00, 0x8d, 0xdd),
            background_light: palette.background_light,
            background_dark: palette.background_dark,
            foreground_light: Color::rgb8(0xf9, 0xf9, 0xf9),
            foreground_dark: Color::rgb8(0xbf, 0xbf, 0xbf),
            disabled_foreground_light: Color::rgb8(0x89, 0x89, 0x89),
            disabled_foreground_dark: Color::rgb8(0x6f, 0x6f, 0x6f),
            button_dark: palette.button_dark,
            button_light: palette.button_light,
            disabled_button_dark: Color::grey8(0x28),
            disabled_button_light: Color::grey8(0x38),
            border_dark: Color::rgb8(0x3a, 0x3a, 0x3a),
            border_light: Color::rgb8(0xa1, 0xa1, 0xa1),
            selected_text_background_color: Color::rgb8(0x43, 0x70, 0xA8),
            selected_text_inactive_background_color: Color::grey8(0x74),
            selection_text_color: Color::rgb8(0x00, 0x00, 0x00),
            cursor_color: palette.cursor_color,
            scrollbar_color: Color::rgb8(0xff, 0xff, 0xff),
            scrollbar_border_color: Color::rgb8(0x77, 0x77, 0x77),
            progress_bar_radius: 4.0,
            button_border_radius: 4.0,
            button_border_width: 2.0,
            text_size_normal: 15.0,
            text_size_large: 24.0,
            basic_widget_height: 18.0,
            wide_widget_width: 100.0,
            bordered_widget_height: 24.0,
            textbox_border_radius: 2.0,
            textbox_border_width: 1.0,
            textbox_insets: Insets::new(4.0, 4.0, 4.0, 4.0),
            scrollbar_max_opacity: 0.7,
            scrollbar_fade_delay: 1500u64,
            scrollbar_width: 8.0,
            scrollbar_pad: 2.0,
            scrollbar_min_size: 45.0,
            scrollbar_radius: 5.0,
            scrollbar_edge_width: 1.0,
            widget_padding_vertical: 10.0,
            widget_padding_horizontal: 8.0,
            widget_control_component_padding: 4.0,
            ui_font: FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(15.0),
            ui_font_bold: FontDescriptor::new(FontFamily::SYSTEM_UI)
                .with_weight(FontWeight::BOLD)
                .with_size(15.0),
            ui_font_italic: FontDescriptor::new(FontFamily::SYSTEM_UI)
                .with_style(FontStyle::Italic)
                .with_size(15.0),
        },
        general: GeneralTheme {
            window_background_color: palette.background_color,
            window_border_color: palette.stroke_color,
        },
        main: MainWindowTheme {
            window_background_color: palette.background_color,
            window_border_color: palette.stroke_color,
            item_background_color_hover: palette.highlight_background_color,
            browser_label_font_family: palette.font_family.clone(),
            browser_label_size: palette.browser_label_size,
            browser_label_color: palette.label_color,
            browser_label_color_hover: palette.label_color_hover,
            profile_label_font_family: palette.font_family.clone(),
            profile_label_size: palette.profile_label_size,
            profile_label_color: palette.secondary_label_color,
            profile_label_color_hover: palette.secondary_label_color_hover,
            url_label_font_family: palette.font_family.clone(),
            url_label_size: palette.url_label_size,
            url_label_color: palette.muted_label_color,
            hotkey_background_color: palette.secondary_background_color,
            hotkey_background_color_hover: palette.hotkey_background_color_hover,
            hotkey_border_color: palette.secondary_stroke_color,
            hotkey_text_color: palette.muted_label_color,
            hotkey_text_color_hover: palette.hotkey_text_color_hover,
            options_button_text_color: palette.muted_label_color,
        },
        settings: SettingsWindowTheme {
            active_tab_background_color: palette.accent_color,
            active_tab_text_color: palette.on_accent_color,
            inactive_tab_text_color: palette.label_color,
            rule_background_color: palette.subtle_background_color,
            rule_border_color: palette.stroke_color,
        },
        about: AboutWindowTheme {
            window_background_color: palette.about_background_color,
        },
    }
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
    item_background_color_hover: Color,
    browser_label_font_family: FontFamily,
    browser_label_size: f64,
    browser_label_color: Color,
    browser_label_color_hover: Color,
    profile_label_font_family: FontFamily,
    profile_label_size: f64,
    profile_label_color: Color,
    profile_label_color_hover: Color,
    url_label_font_family: FontFamily,
    url_label_size: f64,
    url_label_color: Color,
    hotkey_background_color: Color,
    hotkey_background_color_hover: Color,
    hotkey_border_color: Color,
    hotkey_text_color: Color,
    hotkey_text_color_hover: Color,
    options_button_text_color: Color,
}

impl MainWindowTheme {
    pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_background_color");

    pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.window_border_color");

    pub const ENV_ITEM_BACKGROUND_COLOR_HOVER: Key<Color> =
        Key::new("software.browsers.theme.main.item_background_color_hover");

    pub const ENV_BROWSER_LABEL_FONT_FAMILY: Key<FontDescriptor> =
        Key::new("software.browsers.theme.main.browser_label_font_family");

    pub const ENV_BROWSER_LABEL_SIZE: Key<f64> =
        Key::new("software.browsers.theme.main.browser_label_size");

    pub const ENV_BROWSER_LABEL_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.browser_label_color");

    pub const ENV_BROWSER_LABEL_COLOR_HOVER: Key<Color> =
        Key::new("software.browsers.theme.main.browser_label_color_hover");

    pub const ENV_PROFILE_LABEL_FONT_FAMILY: Key<FontDescriptor> =
        Key::new("software.browsers.theme.main.profile_label_font_family");

    pub const ENV_PROFILE_LABEL_SIZE: Key<f64> =
        Key::new("software.browsers.theme.main.profile_label_size");

    pub const ENV_PROFILE_LABEL_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.profile_label_color");

    pub const ENV_PROFILE_LABEL_COLOR_HOVER: Key<Color> =
        Key::new("software.browsers.theme.main.profile_label_color_hover");

    pub const ENV_URL_LABEL_FONT_FAMILY: Key<FontDescriptor> =
        Key::new("software.browsers.theme.main.url_label_font_family");

    pub const ENV_URL_LABEL_SIZE: Key<f64> =
        Key::new("software.browsers.theme.main.url_label_size");

    pub const ENV_URL_LABEL_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.url_label_color");

    pub const ENV_HOTKEY_BACKGROUND_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_background_color");

    pub const ENV_HOTKEY_BACKGROUND_COLOR_HOVER: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_background_color_hover");

    pub const ENV_HOTKEY_BORDER_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_border_color");

    pub const ENV_HOTKEY_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_text_color");

    pub const ENV_HOTKEY_TEXT_COLOR_HOVER: Key<Color> =
        Key::new("software.browsers.theme.main.hotkey_text_color_hover");

    pub const ENV_OPTIONS_BUTTON_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.theme.main.options_button_text_color");

    fn set_env_to_theme(&self, env: &mut Env) {
        env.set(Self::ENV_WINDOW_BACKGROUND_COLOR, self.window_background_color);
        env.set(Self::ENV_WINDOW_BORDER_COLOR, self.window_border_color);
        env.set(
            Self::ENV_ITEM_BACKGROUND_COLOR_HOVER,
            self.item_background_color_hover,
        );
        env.set(
            Self::ENV_BROWSER_LABEL_FONT_FAMILY,
            FontDescriptor::new(self.browser_label_font_family.clone()),
        );
        env.set(Self::ENV_BROWSER_LABEL_SIZE, self.browser_label_size);
        env.set(Self::ENV_BROWSER_LABEL_COLOR, self.browser_label_color);
        env.set(
            Self::ENV_BROWSER_LABEL_COLOR_HOVER,
            self.browser_label_color_hover,
        );
        env.set(
            Self::ENV_PROFILE_LABEL_FONT_FAMILY,
            FontDescriptor::new(self.profile_label_font_family.clone()),
        );
        env.set(Self::ENV_PROFILE_LABEL_SIZE, self.profile_label_size);
        env.set(Self::ENV_PROFILE_LABEL_COLOR, self.profile_label_color);
        env.set(
            Self::ENV_PROFILE_LABEL_COLOR_HOVER,
            self.profile_label_color_hover,
        );
        env.set(
            Self::ENV_URL_LABEL_FONT_FAMILY,
            FontDescriptor::new(self.url_label_font_family.clone()),
        );
        env.set(Self::ENV_URL_LABEL_SIZE, self.url_label_size);
        env.set(Self::ENV_URL_LABEL_COLOR, self.url_label_color);
        env.set(Self::ENV_HOTKEY_BACKGROUND_COLOR, self.hotkey_background_color);
        env.set(
            Self::ENV_HOTKEY_BACKGROUND_COLOR_HOVER,
            self.hotkey_background_color_hover,
        );
        env.set(Self::ENV_HOTKEY_BORDER_COLOR, self.hotkey_border_color);
        env.set(Self::ENV_HOTKEY_TEXT_COLOR, self.hotkey_text_color);
        env.set(Self::ENV_HOTKEY_TEXT_COLOR_HOVER, self.hotkey_text_color_hover);
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
    progress_bar_radius: f64,
    button_border_radius: f64,
    button_border_width: f64,
    text_size_normal: f64,
    text_size_large: f64,
    basic_widget_height: f64,
    wide_widget_width: f64,
    bordered_widget_height: f64,
    textbox_border_radius: f64,
    textbox_border_width: f64,
    textbox_insets: Insets,
    scrollbar_max_opacity: f64,
    scrollbar_fade_delay: u64,
    scrollbar_width: f64,
    scrollbar_pad: f64,
    scrollbar_min_size: f64,
    scrollbar_radius: f64,
    scrollbar_edge_width: f64,
    widget_padding_vertical: f64,
    widget_padding_horizontal: f64,
    widget_control_component_padding: f64,
    ui_font: FontDescriptor,
    ui_font_bold: FontDescriptor,
    ui_font_italic: FontDescriptor,
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

        env.set(druid::theme::PROGRESS_BAR_RADIUS, self.progress_bar_radius);
        env.set(druid::theme::BUTTON_BORDER_RADIUS, self.button_border_radius);
        env.set(druid::theme::BUTTON_BORDER_WIDTH, self.button_border_width);
        env.set(druid::theme::TEXT_SIZE_NORMAL, self.text_size_normal);
        env.set(druid::theme::TEXT_SIZE_LARGE, self.text_size_large);
        env.set(druid::theme::BASIC_WIDGET_HEIGHT, self.basic_widget_height);
        env.set(druid::theme::WIDE_WIDGET_WIDTH, self.wide_widget_width);
        env.set(druid::theme::BORDERED_WIDGET_HEIGHT, self.bordered_widget_height);
        env.set(druid::theme::TEXTBOX_BORDER_RADIUS, self.textbox_border_radius);
        env.set(druid::theme::TEXTBOX_BORDER_WIDTH, self.textbox_border_width);
        env.set(druid::theme::TEXTBOX_INSETS, self.textbox_insets);
        env.set(druid::theme::SCROLLBAR_MAX_OPACITY, self.scrollbar_max_opacity);
        env.set(druid::theme::SCROLLBAR_FADE_DELAY, self.scrollbar_fade_delay);
        env.set(druid::theme::SCROLLBAR_WIDTH, self.scrollbar_width);
        env.set(druid::theme::SCROLLBAR_PAD, self.scrollbar_pad);
        env.set(druid::theme::SCROLLBAR_MIN_SIZE, self.scrollbar_min_size);
        env.set(druid::theme::SCROLLBAR_RADIUS, self.scrollbar_radius);
        env.set(druid::theme::SCROLLBAR_EDGE_WIDTH, self.scrollbar_edge_width);
        env.set(
            druid::theme::WIDGET_PADDING_VERTICAL,
            self.widget_padding_vertical,
        );
        env.set(
            druid::theme::WIDGET_PADDING_HORIZONTAL,
            self.widget_padding_horizontal,
        );
        env.set(
            druid::theme::WIDGET_CONTROL_COMPONENT_PADDING,
            self.widget_control_component_padding,
        );
        env.set(druid::theme::UI_FONT, self.ui_font.clone());
        env.set(druid::theme::UI_FONT_BOLD, self.ui_font_bold.clone());
        env.set(druid::theme::UI_FONT_ITALIC, self.ui_font_italic.clone());
    }
}

struct Palette {
    background_color: Color,
    label_color: Color,
    secondary_label_color: Color,
    muted_label_color: Color,
    stroke_color: Color,
    secondary_stroke_color: Color,
    highlight_background_color: Color,
    secondary_background_color: Color,
    subtle_background_color: Color,
    accent_color: Color,
    on_accent_color: Color,
    opaque_background_color: Color,
    text_color: Color,
    background_light: Color,
    background_dark: Color,
    button_dark: Color,
    button_light: Color,
    cursor_color: Color,
    about_background_color: Color,
    font_family: FontFamily,
    browser_label_size: f64,
    profile_label_size: f64,
    url_label_size: f64,
    label_color_hover: Color,
    secondary_label_color_hover: Color,
    hotkey_background_color_hover: Color,
    hotkey_text_color_hover: Color,
}

impl Palette {
    fn dark() -> Self {
        Palette {
            background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
            label_color: Color::rgb8(255, 255, 255),
            secondary_label_color: Color::rgb8(190, 190, 190),
            muted_label_color: Color::rgb8(128, 128, 128),
            stroke_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
            secondary_stroke_color: Color::rgba(0.4, 0.4, 0.4, 0.9),
            highlight_background_color: Color::rgba(1.0, 1.0, 1.0, 0.25),
            secondary_background_color: Color::rgba(0.15, 0.15, 0.15, 1.0),
            subtle_background_color: Color::rgba(0.1, 0.1, 0.1, 0.9),
            accent_color: Color::rgb8(25, 90, 194),
            on_accent_color: Color::rgb8(255, 255, 255),
            opaque_background_color: Color::rgb8(0x29, 0x29, 0x29),
            text_color: Color::rgb8(0xf0, 0xf0, 0xea),
            background_light: Color::rgb8(0x3a, 0x3a, 0x3a),
            background_dark: Color::rgb8(0x31, 0x31, 0x31),
            button_dark: Color::BLACK,
            button_light: Color::rgb8(0x21, 0x21, 0x21),
            cursor_color: Color::WHITE,
            about_background_color: Color::rgb8(27, 32, 32),
            font_family: FontFamily::SYSTEM_UI,
            browser_label_size: 12.0,
            profile_label_size: 11.0,
            url_label_size: 12.0,
            label_color_hover: Color::rgb8(255, 255, 255),
            secondary_label_color_hover: Color::rgb8(255, 255, 255),
            hotkey_background_color_hover: Color::rgba(0.15, 0.15, 0.15, 1.0),
            hotkey_text_color_hover: Color::rgb8(255, 255, 255),
        }
    }

    fn light() -> Self {
        Palette {
            background_color: Color::rgba(0.85, 0.85, 0.85, 0.9),
            label_color: Color::rgb8(0, 0, 0),
            secondary_label_color: Color::rgb8(30, 30, 30),
            muted_label_color: Color::rgb8(128, 128, 128),
            stroke_color: Color::rgba(0.7, 0.7, 0.7, 0.9),
            secondary_stroke_color: Color::rgba(0.4, 0.4, 0.4, 0.9),
            highlight_background_color: Color::rgba(1.0, 1.0, 1.0, 0.25),
            secondary_background_color: Color::rgb8(215, 215, 215),
            subtle_background_color: Color::rgba(0.8, 0.8, 0.8, 0.9),
            accent_color: Color::rgb8(25, 90, 194),
            on_accent_color: Color::rgb8(255, 255, 255),
            opaque_background_color: Color::rgb(0.85, 0.85, 0.85),
            text_color: Color::rgb8(10, 10, 10),
            background_light: Color::rgb8(220, 220, 220),
            background_dark: Color::rgb8(200, 200, 200),
            button_dark: Color::rgb8(120, 120, 120),
            button_light: Color::rgb8(150, 150, 150),
            cursor_color: Color::BLACK,
            about_background_color: Color::rgb8(236, 236, 236),
            font_family: FontFamily::SYSTEM_UI,
            browser_label_size: 12.0,
            profile_label_size: 11.0,
            url_label_size: 12.0,
            label_color_hover: Color::rgb8(0, 0, 0),
            secondary_label_color_hover: Color::rgb8(0, 0, 0),
            hotkey_background_color_hover: Color::rgb8(215, 215, 215),
            hotkey_text_color_hover: Color::rgb8(0, 0, 0),
        }
    }

    fn from_custom(custom: &CustomPalette) -> Self {
        let fallback = Palette::dark();
        Palette {
            background_color: parse_color(&custom.background_color, fallback.background_color),
            label_color: parse_color(&custom.label_color, fallback.label_color),
            secondary_label_color: parse_color(
                &custom.secondary_label_color,
                fallback.secondary_label_color,
            ),
            muted_label_color: parse_color(&custom.muted_label_color, fallback.muted_label_color),
            stroke_color: parse_color(&custom.stroke_color, fallback.stroke_color),
            secondary_stroke_color: parse_color(
                &custom.secondary_stroke_color,
                fallback.secondary_stroke_color,
            ),
            highlight_background_color: parse_color(
                &custom.highlight_background_color,
                fallback.highlight_background_color,
            ),
            secondary_background_color: parse_color(
                &custom.secondary_background_color,
                fallback.secondary_background_color,
            ),
            subtle_background_color: parse_color(
                &custom.subtle_background_color,
                fallback.subtle_background_color,
            ),
            accent_color: parse_color(&custom.accent_color, fallback.accent_color),
            on_accent_color: parse_color(&custom.on_accent_color, fallback.on_accent_color),
            opaque_background_color: parse_color(
                &custom.opaque_background_color,
                fallback.opaque_background_color,
            ),
            text_color: parse_color(&custom.text_color, fallback.text_color),
            background_light: parse_color(&custom.background_light, fallback.background_light),
            background_dark: parse_color(&custom.background_dark, fallback.background_dark),
            button_dark: parse_color(&custom.button_dark, fallback.button_dark),
            button_light: parse_color(&custom.button_light, fallback.button_light),
            cursor_color: parse_color(&custom.cursor_color, fallback.cursor_color),
            about_background_color: parse_color(
                &custom.about_background_color,
                fallback.about_background_color,
            ),
            font_family: parse_font_family(&custom.font_family),
            browser_label_size: parse_text_size(
                custom.browser_label_size,
                fallback.browser_label_size,
            ),
            profile_label_size: parse_text_size(
                custom.profile_label_size,
                fallback.profile_label_size,
            ),
            url_label_size: parse_text_size(custom.url_label_size, fallback.url_label_size),
            label_color_hover: parse_color(&custom.label_color_hover, fallback.label_color_hover),
            secondary_label_color_hover: parse_color(
                &custom.secondary_label_color_hover,
                fallback.secondary_label_color_hover,
            ),
            hotkey_background_color_hover: parse_color(
                &custom.hotkey_background_color_hover,
                fallback.hotkey_background_color_hover,
            ),
            hotkey_text_color_hover: parse_color(
                &custom.hotkey_text_color_hover,
                fallback.hotkey_text_color_hover,
            ),
        }
    }
}

fn parse_color(hex: &str, fallback: Color) -> Color {
    Color::from_hex_str(hex).unwrap_or_else(|error| {
        warn!("invalid custom palette color '{}': {:?}", hex, error);
        fallback
    })
}

fn parse_font_family(name: &str) -> FontFamily {
    match name.trim().to_lowercase().as_str() {
        "system-ui" | "" => FontFamily::SYSTEM_UI,
        "serif" => FontFamily::SERIF,
        "sans-serif" => FontFamily::SANS_SERIF,
        "monospace" => FontFamily::MONOSPACE,
        _ => FontFamily::new_unchecked(name.trim()),
    }
}

fn parse_text_size(size: f64, fallback: f64) -> f64 {
    if size.is_finite() && size > 0.0 {
        size
    } else {
        warn!("invalid custom palette text size '{}'", size);
        fallback
    }
}
