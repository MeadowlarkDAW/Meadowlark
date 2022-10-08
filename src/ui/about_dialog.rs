use gtk::gio::SimpleAction;
use gtk::glib::{self, clone};
use gtk::prelude::*;

pub fn setup(window: &gtk::ApplicationWindow) {
    let action_open_about_dialog = SimpleAction::new("open-about-dialog", None);
    action_open_about_dialog.connect_activate(clone!(@weak window => move |_, _| {
        let about_dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .title("About Meadowlark")
            .logo(&gtk::gdk::Texture::from_resource("/app/meadowlark/Meadowlark/icons/256x256/apps/meadowlark-logo-256.png"))
            .program_name("Meadowlark")
            .version("alpha-alpha-alpha")
            .authors(vec!["Billy Messenger".into()])
            .license_type(gtk::License::Gpl30)
            .website("https://meadowlark.app")
            .comments("Meadowlark is a (currently incomplete) project that aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.")
            .destroy_with_parent(true)
            .build();

        about_dialog.show();
    }));
    window.add_action(&action_open_about_dialog);
}
