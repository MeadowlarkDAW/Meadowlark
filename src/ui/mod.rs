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

fn setup_style() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_bytes!("resources/styles/default.gcss"));
    gtk::StyleContext::add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &gtk::Application) {
    setup_style();

    let main_window_menu = main_window_menu_bar::setup();
    let header_bar = gtk::HeaderBar::new();
    header_bar.pack_start(&main_window_menu);

    let main_window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Meadowlark")
        .width_request(1280)
        .height_request(800)
        .titlebar(&header_bar)
        .show_menubar(true)
        .build();

    about_dialog::setup(&main_window);

    // Present the window
    main_window.present();
}
