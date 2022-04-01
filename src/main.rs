// On Windows platform, don't show a console when opening the app.
// #![windows_subsystem = "windows"]

mod backend;
mod state;
mod ui;
mod util;

fn main() -> Result<(), String> {
    backend::cpu_id::init();

    //Initiate a simple logger that logs all events to the console

    //TODO: Use something more sophisticated
    //simple_logger::SimpleLogger::new().init().unwrap();

    let config = simple_log::LogConfigBuilder::builder()
        .path("./log/output.log")
        .size(1 * 100)
        .roll_count(10)
        .time_format("%Y-%m-%d %H:%M:%S.%f") //E.g:%H:%M:%S.%f
        .level("debug")
        //.output_file()
        .output_console()
        .build();

    simple_log::new(config)?;

    ui::run()
}
