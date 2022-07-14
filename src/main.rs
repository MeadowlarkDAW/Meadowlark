// TODO: Remove these
#![allow(unused_variables)]
#![allow(dead_code)]

use log::LevelFilter;
use std::error::Error;

mod backend;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging()?;

    ui::run_ui()
}

fn setup_logging() -> Result<(), Box<dyn Error>> {
    use fern::colors::ColoredLevelConfig;

    #[cfg(debug_assertions)]
    const MAIN_LOG_LEVEL: LevelFilter = LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    const MAIN_LOG_LEVEL: LevelFilter = LevelFilter::Info;

    let colors = ColoredLevelConfig::default();

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        // Add blanket level filter -
        .level(MAIN_LOG_LEVEL)
        // Symphonia is quite spammy with its logging
        .level_for("symphonia_core", LevelFilter::Warn)
        .level_for("symphonia_bundle_mp3", LevelFilter::Warn)
        .level_for("symphonia_format_ogg", LevelFilter::Warn)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        // Apply globally
        .apply()?;

    Ok(())
}
