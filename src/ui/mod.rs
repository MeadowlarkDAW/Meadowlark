use gtk::prelude::*;
use std::error::Error;

const APP_TITLE: &str = "Meadowlark";
const APP_ID: &str = "app.meadowlark.Meadowlark";

const MEADOWLARK_ICONS_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark-Icons.ttf");
const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();

    Ok(())
}

fn build_ui(app: &gtk::Application) {
    // Create a window
    let window = gtk::ApplicationWindow::builder().application(app).title(APP_TITLE).build();

    // Present the window
    window.present();
}
