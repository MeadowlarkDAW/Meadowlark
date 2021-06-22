mod frontend;
mod rt_backend;
mod ui;

fn main() {
    // Initiate a simple logger that logs all events to the console
    //
    // TODO: Use something more sophisticated
    simple_logger::SimpleLogger::new().init().unwrap();

    let (frontend_state, backend_state) = frontend::FrontendState::new();

    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let _stream = rt_backend::run_with_default_output(backend_state);

    ui::run(frontend_state);
}
