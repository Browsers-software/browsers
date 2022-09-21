use std::fs;

use rolling_file;
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use browsers::basically_main;
use browsers::paths;

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

    basically_main();
}
