use std::sync::Arc;

use druid::lens::Identity;
use druid::widget::{Button, Checkbox, Flex, Label, List, TextBox};
use druid::{
    Color, DelegateCtx, EventCtx, LensExt, Menu, MenuItem, Point, Widget, WidgetExt, WindowDesc,
};
use tracing::info;

use crate::gui::ui::{UIBrowser, UISettings, UISettingsRule, UIState};

fn create_rule(browsers: &Arc<Vec<UIBrowser>>) -> impl Widget<(UISettingsRule)> {
    let url_pattern_label = Label::new("If URL contains");
    let profile_label = Label::new("Open in");

    let url_pattern = TextBox::new()
        .with_placeholder("https://")
        .with_text_size(18.0)
        .fix_width(200.0)
        .lens(UISettingsRule::url_pattern);

    let browsers_clone = browsers.clone();
    let browsers_clone2 = browsers.clone();
    let selected_profile = Label::dynamic(move |rule: &UISettingsRule, _| {
        let profile_maybe = browsers_clone
            .iter()
            .filter(|b| b.unique_id == rule.profile.clone())
            .map(|b| b.get_full_name())
            .next();
        let profile_name = profile_maybe.unwrap_or("Unknown".to_string());

        format!("{profile_name} ▼")
    })
    .border(Color::rgba(0.5, 0.5, 0.5, 0.9), 0.5)
    .on_click(move |ctx: &mut EventCtx, rule: &mut UISettingsRule, _env| {
        // Windows requires exact position relative to the window
        /*let position = Point::new(
            window_size.width
                - crate::gui::ui::PADDING_X
                - crate::gui::ui::OPTIONS_LABEL_SIZE / 2.0,
            window_size.height
                - crate::gui::ui::PADDING_Y
                - crate::gui::ui::OPTIONS_LABEL_SIZE / 2.0,
        );*/

        ctx.show_context_menu(
            make_profiles_menu(browsers_clone2.clone(), rule),
            Point::new(0.0, 0.0),
        );
    });

    let url_pattern_row = Flex::row()
        .with_child(url_pattern_label)
        .with_child(url_pattern);

    let incognito_checkbox = Checkbox::new("incognito").lens(UISettingsRule::incognito);

    let profile_row = Flex::row()
        .with_child(profile_label)
        .with_child(selected_profile)
        .with_child(incognito_checkbox);
    return Flex::column()
        .with_child(url_pattern_row)
        .with_child(profile_row);
}

pub fn show_settings_dialog(ctx: &mut DelegateCtx, browsers: &Arc<Vec<UIBrowser>>) {
    info!("show_settings_dialog");

    let app_name_row: Label<UIState> = Label::new("Rules");
    let browsers_arc = browsers.clone();

    let rules_list = List::new(move || create_rule(&browsers_arc))
        .with_spacing(20.0)
        .lens(UIState::ui_settings.then(UISettings::rules))
        .scroll();

    let add_rule_button = Button::new("Add rule")
        .on_click(move |_ctx, data: &mut UISettings, _env| data.add_empty_rule())
        .lens(UIState::ui_settings);

    let col = Flex::column()
        .with_child(app_name_row)
        .with_child(rules_list)
        .with_child(add_rule_button);

    let new_win = WindowDesc::new(col).title("Settings").show_titlebar(true);

    /*
    Default browser : pick profile from dropdown or pick Prompt

    Rules: url pattern, browser (profile) picker, incognito flag
        For url pattern could have fully gui options for wildcards (or allow user to use *).
        Maybe switch between advanced and novice.
        Should detect if pattern is yoo complex then only advanced.
        Maybe have the advanced/novice option per rule.
     */
    ctx.new_window(new_win);
}

fn make_profiles_menu(browsers: Arc<Vec<UIBrowser>>, rule: &mut UISettingsRule) -> Menu<UIState> {
    // TODO: should also add Prompt (no profile) as an option (also in settings).
    let menu = browsers
        .iter()
        .map(move |b| {
            let rule_index = rule.index.clone();
            let profile_id = b.unique_id.clone();

            MenuItem::new(b.get_full_name())
                .selected(b.unique_id == rule.profile.to_string())
                .on_activate(move |ctx, data: &mut UISettingsRule, _env| {
                    data.profile = profile_id.clone();
                })
                .lens(
                    UIState::ui_settings
                        .then(UISettings::rules)
                        .then(Identity.index(rule_index).in_arc()),
                )
        })
        .fold(Menu::empty(), |acc, e| acc.entry(e));

    menu
}
