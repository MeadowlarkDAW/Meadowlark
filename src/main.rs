pub mod cpu_id;

mod frontend_state;
mod graph_state;
mod rt_backend;
mod ui;

fn main() {
    cpu_id::init();

    // Initiate a simple logger that logs all events to the console
    //
    // TODO: Use something more sophisticated
    simple_logger::SimpleLogger::new().init().unwrap();

    ui::run();
}
