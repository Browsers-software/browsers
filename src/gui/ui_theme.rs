use druid::{Color, Data, Env, Key};
use serde::{Deserialize, Serialize};

use crate::gui::ui::UIState;
use crate::utils::ConfiguredTheme;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Data)]
pub enum UITheme {
    Light,
    Dark,
}

pub const ENV_WINDOW_BACKGROUND_COLOR: Key<Color> =
    Key::new("software.browsers.theme.window_background_color");

pub const ENV_WINDOW_BORDER_COLOR: Key<Color> =
    Key::new("software.browsers.theme.window_border_color");

//pub const GREY_000: Key<Color> = Key::new("app.grey_000");

//pub const WINDOW_BACKGROUND_COLOR2: Key<Color> =
//    Key::new("org.linebender.druid.theme.window_background_color");

// A widget can retrieve theme parameters (colors, dimensions, etc.). In addition, it can pass custom data down to all descendants. An important example of the latter is setting a value for enabled/ disabled status so that an entire subtree can be disabled ("grayed out") with one setting.
// EnvScope can be used to override parts of Env for its descendants.

pub fn initialize_theme(env: &mut Env, ui_state: &UIState) {
    let ui_theme: UITheme = match ui_state.ui_settings.visual_settings.theme {
        ConfiguredTheme::Auto => UITheme::Dark,
        ConfiguredTheme::Light => UITheme::Light,
        ConfiguredTheme::Dark => UITheme::Dark,
    };

    setup_theme(env, ui_theme);
}

// mingit eventi veel kus me uuendame env-i

pub fn setup_theme(env: &mut Env, ui_theme: UITheme) {
    let dark_theme = Theme {
        window_background_color: Color::rgba(0.15, 0.15, 0.15, 0.9),
        window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
    };

    let light_theme = Theme {
        window_background_color: Color::rgba(0.85, 0.85, 0.85, 0.9),
        window_border_color: Color::rgba(0.5, 0.5, 0.5, 0.9),
    };

    let theme = match ui_theme {
        UITheme::Light => light_theme,
        UITheme::Dark => dark_theme,
    };

    set_env_to_theme(env, theme);

    // set color palette

    //env.set(druid::theme::WINDOW_BACKGROUND_COLOR, env.get(GREY_700));
}

fn set_env_to_theme(env: &mut Env, theme: Theme) {
    //env.adding()
    env.set(ENV_WINDOW_BACKGROUND_COLOR, theme.window_background_color);
    env.set(ENV_WINDOW_BORDER_COLOR, theme.window_background_color);
}

struct Theme {
    window_background_color: Color,
    window_border_color: Color,
}

struct Palette {}

// .adding(BACKGROUND_DARK, Color::rgb8(0x31, 0x31, 0x31))

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
/*
pub fn setup(env: &mut Env, state: &UIState) {
    match state.config.theme {
        Theme::Light => setup_light_theme(env),
        Theme::Dark => setup_dark_theme(env),
    };

    env.set(WINDOW_BACKGROUND_COLOR, env.get(GREY_700));
    env.set(TEXT_COLOR, env.get(GREY_100));
    env.set(ICON_COLOR, env.get(GREY_400));
    env.set(PLACEHOLDER_COLOR, env.get(GREY_400));
    env.set(PRIMARY_LIGHT, env.get(BLUE_100));
    env.set(PRIMARY_DARK, env.get(BLUE_200));

    env.set(BACKGROUND_LIGHT, env.get(GREY_700));
    env.set(BACKGROUND_DARK, env.get(GREY_600));
    env.set(FOREGROUND_LIGHT, env.get(GREY_100));
    env.set(FOREGROUND_DARK, env.get(GREY_000));

    match state.config.theme {
        Theme::Light => {
            env.set(BUTTON_LIGHT, env.get(GREY_700));
            env.set(BUTTON_DARK, env.get(GREY_600));
        }
        Theme::Dark => {
            env.set(BUTTON_LIGHT, env.get(GREY_600));
            env.set(BUTTON_DARK, env.get(GREY_700));
        }
    }

    env.set(BORDER_LIGHT, env.get(GREY_400));
    env.set(BORDER_DARK, env.get(GREY_500));

    env.set(SELECTED_TEXT_BACKGROUND_COLOR, env.get(BLUE_200));
    env.set(SELECTION_TEXT_COLOR, env.get(GREY_700));

    env.set(CURSOR_COLOR, env.get(GREY_000));

    env.set(PROGRESS_BAR_RADIUS, 4.0);
    env.set(BUTTON_BORDER_RADIUS, 4.0);
    env.set(BUTTON_BORDER_WIDTH, 1.0);

    env.set(
        UI_FONT,
        FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(13.0),
    );
    env.set(
        UI_FONT_MEDIUM,
        FontDescriptor::new(FontFamily::SYSTEM_UI)
            .with_size(13.0)
            .with_weight(FontWeight::MEDIUM),
    );
    env.set(
        UI_FONT_MONO,
        FontDescriptor::new(FontFamily::MONOSPACE).with_size(13.0),
    );
    env.set(TEXT_SIZE_SMALL, 11.0);
    env.set(TEXT_SIZE_NORMAL, 13.0);
    env.set(TEXT_SIZE_LARGE, 16.0);

    env.set(BASIC_WIDGET_HEIGHT, 16.0);
    env.set(WIDE_WIDGET_WIDTH, grid(12.0));
    env.set(BORDERED_WIDGET_HEIGHT, grid(4.0));

    env.set(TEXTBOX_BORDER_RADIUS, 4.0);
    env.set(TEXTBOX_BORDER_WIDTH, 1.0);
    env.set(TEXTBOX_INSETS, Insets::uniform_xy(grid(1.2), grid(1.0)));

    env.set(SCROLLBAR_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_BORDER_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_MAX_OPACITY, 0.8);
    env.set(SCROLLBAR_FADE_DELAY, 1500u64);
    env.set(SCROLLBAR_WIDTH, 6.0);
    env.set(SCROLLBAR_PAD, 2.0);
    env.set(SCROLLBAR_RADIUS, 5.0);
    env.set(SCROLLBAR_EDGE_WIDTH, 1.0);

    env.set(WIDGET_PADDING_VERTICAL, grid(0.5));
    env.set(WIDGET_PADDING_HORIZONTAL, grid(1.0));
    env.set(WIDGET_CONTROL_COMPONENT_PADDING, grid(1.0));

    env.set(MENU_BUTTON_BG_ACTIVE, env.get(GREY_500));
    env.set(MENU_BUTTON_BG_INACTIVE, env.get(GREY_600));
    env.set(MENU_BUTTON_FG_ACTIVE, env.get(GREY_000));
    env.set(MENU_BUTTON_FG_INACTIVE, env.get(GREY_100));
}

fn setup_light_theme(env: &mut Env) {
    setup_dark_theme(env);
}

fn setup_dark_theme(env: &mut Env) {
    env.set(GREY_000, Color::grey8(0xff));
    env.set(GREY_100, Color::grey8(0xf2));
    env.set(GREY_200, Color::grey8(0xe0));
    env.set(GREY_300, Color::grey8(0xbd));
    env.set(GREY_400, Color::grey8(0x82));
    env.set(GREY_500, Color::grey8(0x4f));
    env.set(GREY_600, Color::grey8(0x33));
    env.set(GREY_700, Color::grey8(0x28));
    env.set(BLUE_100, Color::rgb8(0x00, 0x8d, 0xdd));
    env.set(BLUE_200, Color::rgb8(0x5c, 0xc4, 0xff));

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.05));
    env.set(LINK_ACTIVE_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.025));
    env.set(LINK_COLD_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.0));
}
*/
