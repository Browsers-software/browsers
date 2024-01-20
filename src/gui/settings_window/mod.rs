use std::sync::Arc;

use druid::widget::{Controller, CrossAxisAlignment, Flex, Label, MainAxisAlignment, ViewSwitcher};
use druid::{
    Data, DelegateCtx, LensExt, Monitor, Point, Size, Widget, WidgetExt, WindowDesc, WindowLevel,
};
use tracing::info;

use crate::gui::ui::{SettingsTab, UIBrowser, UISettings, UIState};

mod general_view;
mod rules_view;

fn tabs_row() -> impl Widget<UISettings> {
    Flex::row()
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(tab_button("General", SettingsTab::GENERAL))
        .with_default_spacer()
        .with_child(tab_button("Rules", SettingsTab::RULES))
}

fn tab_button(text: &'static str, tab: SettingsTab) -> impl Widget<UISettings> {
    Flex::column()
        .with_default_spacer()
        .with_child(Label::new(text))
        .on_click(move |_ctx, data: &mut UISettings, _env| {
            data.tab = tab;
        })
}

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
    let browsers_arc = browsers.clone();

    let view_switcher = ViewSwitcher::new(
        |data: &UISettings, _env| data.clone(),
        move |selector, _data, _env| match selector.tab {
            SettingsTab::GENERAL => Box::new(general_view::general_content()),
            SettingsTab::RULES => Box::new(rules_view::rules_content(browsers_arc.clone())),
        },
    );

    let col = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_child(tabs_row())
        .with_flex_child(view_switcher, 1.0)
        .lens(UIState::ui_settings);

    let size = Size::new(500.0, 400.0);
    let screen_rect = monitor.virtual_work_rect();

    let x = screen_rect.x0 + (screen_rect.x1 - screen_rect.x0) / 2.0 - size.width / 2.0;
    let y = screen_rect.y0 + 190.0;
    let window_position = Point::new(x, y);

    let new_win = WindowDesc::new(col)
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
    return new_win;
}
