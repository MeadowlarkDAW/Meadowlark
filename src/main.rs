mod rt_thread;
mod system_io;
mod ui;

fn main() {
    // Initiate a simple logger that logs all events to the console
    //
    // TODO: Use something more sophisticated
    simple_logger::SimpleLogger::new().init().unwrap();

    ui::run();
}
