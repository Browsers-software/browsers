#![windows_subsystem = "windows"]

use rolling_file;
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use std::str::FromStr;
use std::sync::mpsc;
use std::{env, fs, thread};
use tracing::{Level, info};
use tracing_subscriber;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use browsers::gui::app;
use browsers::gui::app::UiHandle;
use browsers::utils::OSAppFinder;
use browsers::{
    MessageToMain, UrlOpenContext, generate_all_browser_profiles, get_opening_rules,
    open_link_if_matching_rule, prepare_ui, print_visible_options, unwrap_url, utils,
};
use browsers::{handle_messages_to_main, paths};

fn main() {
    // prefer Skia over Slint's default femtovg on macOS - femtovg's lack of font hinting causes
    // small-text artifacts, Skia delegates to CoreText instead (see Cargo.toml for why it's
    // macOS-only).
    // only kicks in if SLINT_BACKEND isn't already set, so it still doubles as an override point
    // (e.g. SLINT_BACKEND=headless for the MCP server)
    #[cfg(target_os = "macos")]
    if env::var_os("SLINT_BACKEND").is_none()
        && let Err(err) = slint::BackendSelector::new()
            .renderer_name("skia".to_string())
            .select()
    {
        eprintln!("Warning: failed to select the Skia renderer, using Slint's default: {err}");
    }

    // see hide_dock_icon's comment for why this is needed alongside Info.plist
    #[cfg(target_os = "macos")]
    browsers::macos::macos_native::hide_dock_icon();

    // has to run before anything else - see register_event_bridge's comment.
    // callbacks get wired up later once state exists, bridge just needs to stay alive till then
    #[cfg(target_os = "macos")]
    let bridge = browsers::macos::macos_native::register_event_bridge();

    let offset_time = OffsetTime::local_rfc_3339().expect("could not get local offset!");

    let logs_root_dir = paths::get_logs_root_dir();
    fs::create_dir_all(logs_root_dir.as_path()).unwrap();

    let log_file_path = logs_root_dir.join("browsers.log");
    let file_appender = BasicRollingFileAppender::new(
        log_file_path.as_path(),
        RollingConditionBasic::new().daily(),
        3,
    )
    .unwrap();

    //let file_appender = tracing_appender::rolling::daily(logs_root_dir, "browsers.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let log_level = env::var("BROWSERS_LOG_LEVEL")
        .ok()
        .and_then(|level| Level::from_str(&level).ok())
        .unwrap_or(Level::INFO);

    if log_level == Level::DEBUG {
        // also show full backtrace if debug log level
        unsafe { env::set_var("RUST_BACKTRACE", "full") };
    }

    tracing_subscriber::fmt()
        .with_timer(offset_time)
        .with_writer(non_blocking.and(std::io::stdout))
        .with_max_level(log_level)
        .with_ansi(false)
        .init();

    info!("Starting Browsers");
    info!("Logging to {}", log_file_path.display());

    let args: Vec<String> = env::args().collect();
    //info!("{:?}", args);

    let mut url = "".to_string();
    let url_input_maybe = args.iter().find(|i| i.starts_with("http"));
    if let Some(url_input) = url_input_maybe {
        url = url_input.to_string();
    }

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

    #[cfg(target_os = "macos")]
    if let Some(bridge) = &bridge {
        let resign_active_state = state.clone();
        bridge.set_resign_active_handler(move || {
            let quits = resign_active_state.borrow().quit_on_lost_focus_applies();
            if quits {
                std::process::exit(0x0100);
            }
        });

        let open_urls_sender = main_sender.clone();
        let open_urls_state = state.clone();
        bridge.set_open_urls_handler(move |sender_bundle_id, url| {
            let unwrap_urls = open_urls_state
                .borrow()
                .ui_settings
                .behavioral_settings
                .unwrap_urls;
            let behavioral_config = utils::BehavioralConfig { unwrap_urls };
            let _ = open_urls_sender.send(MessageToMain::UrlPassedToMain(
                sender_bundle_id.unwrap_or_default(),
                url,
                behavioral_config,
            ));
        });
    }

    let ui_handle = UiHandle::new(&state);
    thread::spawn(move || {
        handle_messages_to_main(
            main_receiver,
            ui_handle,
            &mut opening_rules_and_default_profile,
            &mut visible_and_hidden_profiles,
            &app_finder,
        );
    });

    if open_settings {
        browsers::gui::settings_window::open(&state);
    }
    if open_about {
        browsers::gui::about_window::open(&state);
    }

    app::run(&state);
}
