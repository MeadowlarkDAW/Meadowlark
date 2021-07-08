mod backend;
mod ui;

fn main() {
    backend::cpu_id::init();

    // Initiate a simple logger that logs all events to the console
    //
    // TODO: Use something more sophisticated
    simple_logger::SimpleLogger::new().init().unwrap();

    ui::run();
}
