use crate::gui::app::UiHandle;
use crate::utils::OSAppFinder;
use crate::{
    MessageToMain, UrlOpenContext, generate_all_browser_profiles, get_opening_rules, gui,
    open_link_if_matching_rule, prepare_ui, print_visible_options, unwrap_url, utils,
};
use std::env;
use std::sync::mpsc;
use tracing::info;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn launch_wasm() {
    console_error_panic_hook::set_once();

    use crate::gui::app;

    info!("Starting Browsers");

    let args: Vec<String> = env::args().collect();
    //info!("{:?}", args);

    let mut url = "".to_string();
    let url_input_maybe = args.iter().find(|i| i.starts_with("http"));
    if let Some(url_input) = url_input_maybe {
        url = url_input.to_string();
    }

    let url = "https://github.com/Browsers-software/browsers".to_string();

    let show_gui = !args.contains(&"--no-gui".to_string());
    let force_reload = args.contains(&"--reload".to_string());
    // jump straight to Settings/About, skipping the menu clicks - handy for checking UI changes
    let open_settings = args.contains(&"--settings".to_string());
    let open_about = args.contains(&"--about".to_string());

    let (main_sender, main_receiver) = mpsc::channel::<MessageToMain>();

    let app_finder = OSAppFinder::new();
    let config = app_finder.load_config();
    let mut opening_rules_and_default_profile = get_opening_rules(&config);

    let mut visible_and_hidden_profiles =
        generate_all_browser_profiles(&config, &app_finder, force_reload);

    let behavioral_settings = config.get_behavior();
    // TODO: url should not be considered here in case of macos
    //       and only the one in LinkOpenedFromBundle should be considered
    let cleaned_url = unwrap_url(url.as_str(), behavioral_settings);

    let url_open_context = UrlOpenContext {
        cleaned_url: cleaned_url.clone(),
        source_app_maybe: None,
    };

    if open_link_if_matching_rule(
        &url_open_context,
        &opening_rules_and_default_profile,
        &visible_and_hidden_profiles,
    ) {
        // opened in a browser because of an opening rule, so we are done here
        return;
    }

    let is_default = utils::is_default_web_browser();
    let show_set_as_default = !is_default;

    let prepared_ui = prepare_ui(
        &url_open_context,
        &visible_and_hidden_profiles,
        &config,
        show_set_as_default,
    );

    if !show_gui {
        print_visible_options(&prepared_ui);
        return;
    }

    let state = app::new(
        main_sender.clone(),
        prepared_ui.url,
        prepared_ui.browsers,
        prepared_ui.hidden_browsers,
        prepared_ui.show_set_as_default,
        prepared_ui.ui_settings,
    );

    let ui_handle = UiHandle::new(&state);

    /*
       thread::spawn(move || {
           handle_messages_to_main(
               main_receiver,
               ui_handle,
               &mut opening_rules_and_default_profile,
               &mut visible_and_hidden_profiles,
               &app_finder,
           );
       });
    */

    if open_settings {
        gui::settings_window::open(&state);
    }
    if open_about {
        gui::about_window::open(&state);
    }

    app::run(&state);
}
