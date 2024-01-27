use core::option::Option;
use std::sync::Arc;

use druid::lens::Identity;
use druid::menu::MenuEventCtx;
use druid::widget::{
    Button, Checkbox, Container, Controller, ControllerHost, CrossAxisAlignment, Either, EnvScope,
    Flex, Label, LineBreaking, List, Maybe, TextBox,
};
use druid::{
    Color, Command, Data, Env, EventCtx, FontDescriptor, FontFamily, Key, LensExt, LifeCycle,
    LifeCycleCtx, Menu, MenuItem, Point, UpdateCtx, Widget, WidgetExt,
};

use crate::gui::ui::{
    UIBrowser, UIProfileAndIncognito, UISettings, UISettingsRule, UIState, SAVE_DEFAULT_RULE,
    SAVE_RULE, SAVE_RULES,
};

pub(crate) const FONT: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(12.0);

const RULE_INDEX_KEY: Key<u64> = Key::new("RULE_INDEX");

const CHOOSE_EMPTY_LABEL: &str = "☰ List of Apps";

pub(crate) fn rules_content(browsers: Arc<Vec<UIBrowser>>) -> impl Widget<UIState> {
    let browsers_arc = browsers.clone();
    let browsers_arc2 = browsers.clone();

    // TODO: add default_profile also to rules

    let rules_list = List::new(move || create_rule(&browsers_arc))
        .lens(UISettings::rules)
        .padding((0.0, 0.0, 15.0, 0.0));

    let hint_str = r#"
URL matching examples:
 • github.com/Browsers-software/** starts with "github.com/Browsers-software/"
 • github.com/**/end starts with "github.com/" and ends with "/end"
 • github.com/*/end starts with "github.com/" and ends with "/end" but can have
   only up to one path item in between

See https://github.com/Browsers-software/browsers/wiki/Rules for all the details.
    "#;

    let hint = Label::new(hint_str)
        .with_line_break_mode(LineBreaking::WordWrap)
        .with_text_size(11.0)
        .with_text_color(Color::from_hex_str("808080").unwrap())
        .padding((0.0, 0.0, 15.0, 0.0));

    let rules_list = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(rules_list)
        .with_child(hint)
        .scroll()
        .vertical()
        .content_must_fill(true);

    // viewport size is fixed, while scrollable are is full size
    let rules_list = Container::new(rules_list).expand_height();

    let add_rule_button = Button::from_label(Label::new("Add Rule"))
        .on_click(move |_ctx, data: &mut UISettings, _env| {
            // this will add new entry to data.rules
            // and that triggers rules_list to add new child
            // and that child uses AddRuleController which will then scroll to new rule and save it
            data.add_empty_rule();
        })
        .align_right()
        .padding((10.0, 10.0, 40.0, 0.0));

    let col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(default_app(&browsers_arc2))
        .with_default_spacer()
        .with_flex_child(rules_list, 1.0)
        .with_child(add_rule_button)
        .expand_height();

    return col.lens(UIState::ui_settings);
}

fn create_profile_pop_up_button(
    browsers: &Arc<Vec<UIBrowser>>,
    command: Command,
) -> impl Widget<Option<UIProfileAndIncognito>> + 'static {
    let browsers_clone = browsers.clone();
    let browsers_clone2 = browsers.clone();

    return Label::dynamic(move |opener: &Option<UIProfileAndIncognito>, _| {
        if opener.is_none() {
            let profile_name = CHOOSE_EMPTY_LABEL;
            format!("{profile_name} ▼")
        } else {
            let opener = opener.as_ref().unwrap();
            let browser_maybe = find_browser(&browsers_clone, opener.profile.clone());
            let profile_name_maybe = browser_maybe.map(|b| b.get_full_name());
            let profile_name = profile_name_maybe.unwrap_or("Unknown".to_string());

            format!("{profile_name} ▼")
        }
    })
    .with_font(FONT)
    .border(Color::rgba(0.5, 0.5, 0.5, 0.9), 0.5)
    .on_click(
        move |ctx: &mut EventCtx, _opener: &mut Option<UIProfileAndIncognito>, env: &Env| {
            // Windows requires exact position relative to the window
            /*let position = Point::new(
                window_size.width
                    - crate::gui::ui::PADDING_X
                    - crate::gui::ui::OPTIONS_LABEL_SIZE / 2.0,
                window_size.height
                    - crate::gui::ui::PADDING_Y
                    - crate::gui::ui::OPTIONS_LABEL_SIZE / 2.0,
            );*/

            let rule_index_maybe: Option<usize> =
                env.try_get(RULE_INDEX_KEY.clone()).ok().map(|a| a as usize);

            let menu: Menu<UIState> =
                make_profiles_menu(browsers_clone2.clone(), command.clone(), rule_index_maybe);
            ctx.show_context_menu(menu, Point::new(0.0, 0.0));
        },
    );
}

fn create_incognito_checkbox(
    browsers: &Arc<Vec<UIBrowser>>,
    command: Command,
) -> impl Widget<Option<UIProfileAndIncognito>> {
    let browsers_clone3 = browsers.clone();

    return Maybe::new(
        move || {
            let browsers_clone4 = browsers_clone3.clone();
            let command1 = command.clone();

            let incognito_either = Either::new(
                move |data: &UIProfileAndIncognito, _env| {
                    let browser_maybe = find_browser(&browsers_clone4, data.profile.clone());
                    let browser_supports_incognito_maybe =
                        browser_maybe.map(|p| p.supports_incognito);
                    let profile_supports_incognito =
                        browser_supports_incognito_maybe.unwrap_or(false);
                    profile_supports_incognito
                },
                {
                    let incognito_checkbox = ControllerHost::new(
                        Checkbox::from_label(Label::new("In Incognito").with_font(FONT)),
                        SubmitCommandOnDataChange {
                            command: command1.clone(),
                        },
                    )
                    .lens(UIProfileAndIncognito::incognito)
                    .padding((10.0, 0.0, 0.0, 0.0));
                    incognito_checkbox
                },
                Flex::column(),
            );

            incognito_either
        },
        || Flex::column(),
    );
}

fn create_profile_label() -> Label<Option<UIProfileAndIncognito>> {
    let profile_label = Label::dynamic(|opener, _env| match opener {
        None => "Show".to_string(),
        _ => "Open in".to_string(),
    })
    .with_font(FONT);

    return profile_label;
}

fn default_app(browsers: &Arc<Vec<UIBrowser>>) -> impl Widget<UISettings> {
    let profile_label = create_profile_label();

    let save_profile_command = SAVE_DEFAULT_RULE.with(());
    let selected_profile = create_profile_pop_up_button(browsers, save_profile_command);

    let incognito_save_command = SAVE_DEFAULT_RULE.with(());
    let incognito_maybe = create_incognito_checkbox(browsers, incognito_save_command);

    let profile_row = Flex::row()
        .with_child(profile_label)
        .with_child(selected_profile)
        .with_child(incognito_maybe)
        .padding((0.0, 10.0, 0.0, 0.0));

    return Container::new(
        Flex::row()
            .cross_axis_alignment(CrossAxisAlignment::End)
            .with_child(
                Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(Label::new("If no rules match").with_font(FONT))
                    .with_child(profile_row)
                    .lens(UISettings::default_opener),
            ),
    )
    .padding(10.0)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
    .rounded(10.0)
    .border(Color::rgba(0.5, 0.5, 0.5, 0.9), 0.5)
    .padding((0.0, 5.0, 40.0, 5.0))
    .expand_width();
}

// handles scrolling and saving when Add Rule is pressed
struct AddRuleController;

impl<W: Widget<UISettingsRule>> Controller<UISettingsRule, W> for AddRuleController {
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        rule: &UISettingsRule,
        env: &Env,
    ) {
        if let LifeCycle::ViewContextChanged(_) = event {
            if !rule.deleted && !rule.saved {
                ctx.scroll_to_view();
                ctx.submit_command(SAVE_RULE.with(rule.index));
            }
        }
        child.lifecycle(ctx, event, rule, env)
    }
}

fn create_rule(browsers: &Arc<Vec<UIBrowser>>) -> impl Widget<UISettingsRule> {
    let remove_rule_button = Button::from_label(Label::new("➖").with_text_size(5.0))
        .on_click(move |ctx, data: &mut UISettingsRule, _env| {
            data.deleted = true;
            ctx.submit_command(SAVE_RULES.with(()));
        })
        .fix_size(30.0, 30.0);

    let text_box = TextBox::new()
        .with_placeholder("https://")
        .with_text_size(12.0);

    //let formatter = ParseFormatter::new();
    //let value_text_box = ValueTextBox::new(text_box, formatter).update_data_while_editing(true);
    let value_text_box = ControllerHost::new(
        text_box,
        SubmitCommandOnDataChange {
            command: SAVE_RULES.with(()),
        },
    );

    let url_pattern_label = Label::new("If URL matches").with_font(FONT);
    let url_pattern = value_text_box
        .fix_width(300.0)
        .lens(UISettingsRule::url_pattern);
    let url_pattern_row = Flex::row()
        .with_child(url_pattern_label)
        .with_child(url_pattern);

    let profile_label = create_profile_label().lens(UISettingsRule::opener);

    let save_profile_command = SAVE_RULES.with(());
    let selected_profile = EnvScope::new(
        |env, rule: &UISettingsRule| {
            env.set(RULE_INDEX_KEY.clone(), rule.index as u64);
        },
        create_profile_pop_up_button(browsers, save_profile_command).lens(UISettingsRule::opener),
    );

    let save_incognito_command = SAVE_RULES.with(());
    let incognito_maybe = EnvScope::new(
        |env, rule: &UISettingsRule| {
            env.set(RULE_INDEX_KEY.clone(), rule.index as u64);
        },
        create_incognito_checkbox(browsers, save_incognito_command).lens(UISettingsRule::opener),
    );

    let profile_row = Flex::row()
        .with_child(profile_label)
        .with_child(selected_profile)
        .with_child(incognito_maybe)
        .padding((0.0, 10.0, 0.0, 0.0));

    return Either::new(|data: &UISettingsRule, _env| data.deleted, Flex::column(), {
        Container::new(
            Flex::row()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .with_child(
                    Flex::column()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_child(url_pattern_row)
                        .with_child(profile_row),
                )
                .with_spacer(10.0)
                .with_child(remove_rule_button),
        )
        .padding(10.0)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .rounded(10.0)
        .border(Color::rgba(0.5, 0.5, 0.5, 0.9), 0.5)
        .padding((0.0, 5.0))
    })
    .controller(AddRuleController);
}

fn find_browser(browsers: &Arc<Vec<UIBrowser>>, unique_id: String) -> Option<&UIBrowser> {
    let option = browsers.iter().filter(|b| b.unique_id == unique_id).next();
    return option;
}

pub(crate) struct SubmitCommandOnDataChange {
    pub(crate) command: Command,
}

impl<T: Data, W: Widget<T>> Controller<T, W> for SubmitCommandOnDataChange {
    fn update(&mut self, child: &mut W, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        child.update(ctx, old_data, data, env);
        if !old_data.same(data) {
            ctx.submit_command(self.command.clone());
        }
    }
}

fn item_empty(save_command: Command) -> MenuItem<Option<UIProfileAndIncognito>> {
    let save_command_clone0 = save_command.clone();
    let item = MenuItem::new(CHOOSE_EMPTY_LABEL)
        .selected_if(|opener: &Option<UIProfileAndIncognito>, _env| opener.is_none())
        .on_activate(move |ctx, opener: &mut Option<UIProfileAndIncognito>, _env| {
            *opener = None;
            ctx.submit_command(save_command_clone0.clone())
        });
    return item;
}

fn item_profile(b: &UIBrowser, save_command: Command) -> MenuItem<Option<UIProfileAndIncognito>> {
    let profile_full_name = b.get_full_name();
    let profile_id = b.unique_id.clone();
    let profile_id_clone = profile_id.clone();

    let save_command_clone1 = save_command.clone();
    let save_command_clone2 = save_command_clone1.clone();

    MenuItem::new(profile_full_name)
        .selected_if(move |opener: &Option<UIProfileAndIncognito>, _env| {
            opener.is_some() && opener.as_ref().unwrap().profile == profile_id
        })
        .on_activate(
            move |ctx: &mut MenuEventCtx, opener: &mut Option<UIProfileAndIncognito>, _env| {
                // if it's already an app, then change only the profile
                if opener.is_some() {
                    opener.as_mut().unwrap().profile = profile_id_clone.clone();
                } else {
                    // if it was "<prompt>", then set profile and incognito
                    let option: Option<UIProfileAndIncognito> = Some(UIProfileAndIncognito {
                        profile: profile_id_clone.clone(),
                        incognito: false,
                    });
                    *opener = option;
                }
                ctx.submit_command(save_command_clone2.clone())
            },
        )
}

fn make_profiles_menu(
    browsers: Arc<Vec<UIBrowser>>,
    save_command: Command,
    rule_index_maybe: Option<usize>,
) -> Menu<UIState> {
    let menu_item_empty = item_empty(save_command.clone())
        .lens(UIState::ui_settings.then(UISettings::default_opener));
    let menu_initial = Menu::empty().entry(menu_item_empty);

    let save_command_clone1 = save_command.clone();

    let menu = browsers
        .iter()
        .map(|b| {
            let item = match rule_index_maybe.is_some() {
                true => {
                    let rule_index = rule_index_maybe.unwrap();
                    let ok = UIState::ui_settings
                        .then(UISettings::rules)
                        .then(Identity.index(rule_index).in_arc())
                        .then(UISettingsRule::opener);
                    item_profile(b, save_command_clone1.clone()).lens(ok)
                }
                false => {
                    let ok = UIState::ui_settings.then(UISettings::default_opener);
                    item_profile(b, save_command_clone1.clone()).lens(ok)
                }
            };

            item
        })
        .fold(menu_initial, |acc, e| acc.entry(e));

    menu
}
