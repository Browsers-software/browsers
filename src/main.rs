#![windows_subsystem = "windows"]
use std::sync::mpsc;
use std::{env, fs};

use rolling_file;
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use single_instance::SingleInstance;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use browsers::communicate;
use browsers::paths;
use browsers::{basically_main, MessageToMain};

fn main() {
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

    tracing_subscriber::fmt()
        .with_timer(offset_time)
        .with_writer(non_blocking.and(std::io::stdout))
        .with_max_level(LevelFilter::INFO)
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

    let (main_sender, main_receiver) = mpsc::channel::<MessageToMain>();

    let (is_first_instance, single_instance) =
        communicate::check_single_instance(url.as_str(), main_sender.clone());
    if !is_first_instance {
        info!("Exiting, because another instance is running");
        return;
    }

    basically_main(
        url.as_str(),
        show_gui,
        force_reload,
        main_sender.clone(),
        main_receiver,
    );
    single_instance.is_single(); // dummy as guard
}
