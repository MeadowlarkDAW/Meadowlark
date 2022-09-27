use gtk::prelude::*;
use std::error::Error;

mod about_dialog;
mod main_window_menu_bar;

const APP_ID: &str = "app.meadowlark.Meadowlark";

//const MEADOWLARK_ICONS_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark-Icons.ttf");
//const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();

    Ok(())
}

fn build_ui(app: &gtk::Application) {
    let main_window = gtk::ApplicationWindow::new(app);
    main_window.set_title(Some("Meadowlark"));

    let main_window_menu = main_window_menu_bar::setup();
    let header_bar = gtk::HeaderBar::new();
    header_bar.pack_start(&main_window_menu);
    main_window.set_titlebar(Some(&header_bar));

    about_dialog::setup(&main_window);

    // Present the window
    main_window.present();
}
