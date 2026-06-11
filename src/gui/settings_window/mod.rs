use std::sync::Arc;

use druid::widget::{CrossAxisAlignment, Flex, Label, Painter, ViewSwitcher};
use druid::{
    Color, DelegateCtx, FontDescriptor, FontFamily, FontWeight, Key, LocalizedString, Monitor,
    Point, RenderContext, Size, Widget, WidgetExt, WindowDesc, WindowLevel,
};
use tracing::info;

use crate::gui::ui::{SettingsTab, UIBrowser, UISettings, UIState};
use crate::gui::ui_theme;
use crate::gui::ui_theme::{GeneralTheme, SettingsWindowTheme};

mod advanced_view;
mod appearance_view;
mod general_view;
mod rules_view;

const SIDEBAR_ITEM_WIDTH: f64 = 190.0;
const WINDOW_WIDTH: f64 = 714.0;
const SIDEBAR_PADDING: f64 = 10.0;
const CONTENT_PADDING: f64 = 10.0;
const CONTENT_WIDTH: f64 =
    WINDOW_WIDTH - SIDEBAR_PADDING - SIDEBAR_ITEM_WIDTH - CONTENT_PADDING - 10.0;

pub fn show_settings_dialog(
    ctx: &mut DelegateCtx,
    monitor: Monitor,
    browsers: &Arc<Vec<UIBrowser>>,
) {
    info!("show_settings_dialog");
    let window = create_settings_window(monitor, browsers);
    ctx.new_window(window);
}

pub fn create_settings_window(
    monitor: Monitor,
    browsers: &Arc<Vec<UIBrowser>>,
) -> WindowDesc<UIState> {
    // 210 px
    let sidebar = Flex::column()
        .with_flex_child(sidebar_items(), 1.0)
        .padding(SIDEBAR_PADDING)
        .lens(UIState::ui_settings);

    let content = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(view_switcher(browsers.clone()), 1.0)
        .padding(CONTENT_PADDING);

    let layout = Flex::row().with_child(sidebar).with_child(content);

    let main_column = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(layout, 1.0)
        .background(GeneralTheme::ENV_WINDOW_BACKGROUND_COLOR)
        .env_scope(|env, data| {
            ui_theme::initialize_theme(env, data);
        });

    let size = Size::new(WINDOW_WIDTH, 500.0);
    let screen_rect = monitor.virtual_work_rect();

    let x = screen_rect.x0 + (screen_rect.x1 - screen_rect.x0) / 2.0 - size.width / 2.0;
    let y = screen_rect.y0 + 190.0;
    let window_position = Point::new(x, y);

    return WindowDesc::new(main_column)
        .title("Settings")
        .window_size(size)
        // with_min_size helps on LXDE
        .with_min_size((size.width, 200.0f64))
        // make sure this window is on top of our main window, so using same window level
        // (except that it doesn't work on macOS)
        .set_level(WindowLevel::AppWindow)
        .set_always_on_top(true)
        .show_titlebar(true)
        .resizable(true)
        .set_position(window_position);

    /*
    Rules
        For url pattern could have fully gui options for wildcards (or allow user to use *).
        Maybe switch between advanced and novice.
        Should detect if pattern is too complex then only advanced.
        Maybe have the advanced/novice option per rule.
     */
}

fn view_switcher(browsers_arc: Arc<Vec<UIBrowser>>) -> ViewSwitcher<UIState, SettingsTab> {
    ViewSwitcher::new(
        |data: &UIState, _env| data.ui_settings.tab.clone(),
        move |selector, _data, _env| match selector {
            SettingsTab::GENERAL => {
                settings_view_container("settings-tab-general", general_view::general_content())
            }
            SettingsTab::RULES => settings_view_container(
                "settings-tab-rules",
                rules_view::rules_content(browsers_arc.clone()),
            ),
            SettingsTab::ADVANCED => {
                settings_view_container("settings-tab-advanced", advanced_view::advanced_content())
            }
            SettingsTab::APPEARANCE => settings_view_container(
                "settings-tab-appearance",
                appearance_view::appearance_content(),
            ),
        },
    )
}

fn settings_view_container(
    title_key: &'static str,
    content: impl Widget<UIState> + 'static,
) -> Box<dyn Widget<UIState>> {
    let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(16.0);

    let title = LocalizedString::new(title_key);
    let setting_name_label = Label::new(title).with_font(font).padding((0.0, 10.0));

    // makes sure word-wrapping in content widget is respected
    let content = content
        .padding((0.0, 0.0, 20.0, 0.0))
        .fix_width(CONTENT_WIDTH);

    let col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(setting_name_label)
        .with_flex_child(content, 1.0)
        .expand_height();

    return Box::new(col);
}

fn sidebar_items() -> impl Widget<UISettings> {
    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_child(tab_button("settings-tab-general", SettingsTab::GENERAL))
        .with_child(tab_button("settings-tab-rules", SettingsTab::RULES))
        .with_child(tab_button("settings-tab-appearance", SettingsTab::APPEARANCE))
        .with_child(tab_button("settings-tab-advanced", SettingsTab::ADVANCED))
        .with_flex_spacer(1.0)
        .fix_width(SIDEBAR_ITEM_WIDTH)
}

fn tab_button(key: &'static str, tab: SettingsTab) -> impl Widget<UISettings> {
    let string = LocalizedString::new(key);

    let painter = Painter::new(move |ctx, active_tab, env| {
        if *active_tab == tab {
            let bounds = ctx.size().to_rect();
            ctx.fill(
                bounds,
                &env.get(SettingsWindowTheme::ENV_ACTIVE_TAB_BACKGROUND_COLOR),
            );
        }
    });

    // This is a custom key we'll use with Env to set and get our font.
    const TAB_TEXT_COLOR: Key<Color> =
        Key::new("software.browsers.current.settings.tab_text_color");

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            Label::new(string)
                .with_text_size(14.0)
                .with_text_color(TAB_TEXT_COLOR),
        )
        .on_click(move |_ctx, active_tab: &mut SettingsTab, _env| {
            *active_tab = tab;
        })
        .padding(5.0)
        .background(painter)
        .rounded(5.0)
        .env_scope(move |env: &mut druid::Env, active_tab: &SettingsTab| {
            if tab == *active_tab {
                env.set(
                    TAB_TEXT_COLOR,
                    env.get(SettingsWindowTheme::ENV_ACTIVE_TAB_TEXT_COLOR),
                );

                //env.set(MY_CUSTOM_FONT, new_font);
            } else {
                env.set(
                    TAB_TEXT_COLOR,
                    env.get(SettingsWindowTheme::ENV_INACTIVE_TAB_TEXT_COLOR),
                );
            }
            /* let new_font = if data.mono {
                FontDescriptor::new(FontFamily::MONOSPACE)
            } else {
                FontDescriptor::new(FontFamily::SYSTEM_UI)
            }
            .with_size(data.size);*/
        })
        .lens(UISettings::tab)
}
