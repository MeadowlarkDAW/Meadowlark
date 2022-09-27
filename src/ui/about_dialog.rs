use gtk::gio::SimpleAction;
use gtk::prelude::*;

pub fn setup(window: &gtk::ApplicationWindow) {
    let logo = if let Ok(logo) =
        gtk::gdk::Texture::from_filename("assets/branding/meadowlark-logo-128.png")
    {
        Some(logo)
    } else {
        log::error!("Failed to load icon resource meadowlark-logo-128.png");
        None
    };

    let action_open_about_dialog = SimpleAction::new("open-about-dialog", None);
    let window_clone = window.clone();
    action_open_about_dialog.connect_activate(move |_, _| {
        let about_dialog = gtk::AboutDialog::builder()
            .transient_for(&window_clone)
            .title("About Meadowlark")
            .program_name("Meadowlark")
            .version("alpha-alpha-alpha")
            .authors(vec!["Billy Messenger".into()])
            .license_type(gtk::License::Gpl30)
            .website("https://meadowlark.app")
            .comments("Meadowlark is a (currently incomplete) project that aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.")
            .destroy_with_parent(true)
            .build();

        if let Some(logo) = &logo {
            about_dialog.set_logo(Some(logo));
        }

        about_dialog.show();
    });
    window.add_action(&action_open_about_dialog);
}
