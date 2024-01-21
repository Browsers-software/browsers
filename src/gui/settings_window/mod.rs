use std::sync::Arc;

use druid::widget::{CrossAxisAlignment, Flex, Label, Painter, ViewSwitcher};
use druid::{
    Color, DelegateCtx, FontDescriptor, FontFamily, FontWeight, LocalizedString, Monitor, Point,
    RenderContext, Size, Widget, WidgetExt, WindowDesc, WindowLevel,
};
use tracing::info;

use crate::gui::ui::{SettingsTab, UIBrowser, UISettings, UIState};

mod advanced_view;
mod general_view;
mod rules_view;

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
    let sidebar = Flex::column()
        .with_flex_child(sidebar_items(), 1.0)
        .padding(10.0);

    let content = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(view_switcher(browsers.clone()), 1.0)
        .padding(10.0);

    let layout = Flex::row().with_child(sidebar).with_child(content);

    let main_column = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(layout, 1.0)
        .lens(UIState::ui_settings);

    let size = Size::new(714.0, 500.0);
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
        .show_titlebar(true)
        .resizable(true)
        .set_position(window_position);

    /*
    Default browser : pick profile from dropdown or pick Prompt

    Rules: url pattern, browser (profile) picker, incognito flag
        For url pattern could have fully gui options for wildcards (or allow user to use *).
        Maybe switch between advanced and novice.
        Should detect if pattern is yoo complex then only advanced.
        Maybe have the advanced/novice option per rule.
     */
}

fn view_switcher(browsers_arc: Arc<Vec<UIBrowser>>) -> ViewSwitcher<UISettings, SettingsTab> {
    ViewSwitcher::new(
        |data: &UISettings, _env| data.tab.clone(),
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
        },
    )
}

fn settings_view_container(
    title_key: &'static str,
    content: impl Widget<UISettings> + 'static,
) -> Box<dyn Widget<UISettings>> {
    let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(16.0);

    let title = LocalizedString::new(title_key);
    let setting_name_label = Label::new(title).with_font(font).padding((0.0, 10.0));

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
        .with_child(tab_button("settings-tab-advanced", SettingsTab::ADVANCED))
        .with_flex_spacer(1.0)
        .fix_width(190.0)
}

fn tab_button(key: &'static str, tab: SettingsTab) -> impl Widget<UISettings> {
    let string = LocalizedString::new(key);

    let painter = Painter::new(move |ctx, active_tab, env| {
        if *active_tab == tab {
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &Color::rgb8(25, 90, 194));
        }
    });

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(string).with_text_size(14.0))
        .on_click(move |_ctx, active_tab: &mut SettingsTab, _env| {
            *active_tab = tab;
        })
        .padding(5.0)
        .background(painter)
        .rounded(5.0)
        .lens(UISettings::tab)
}
