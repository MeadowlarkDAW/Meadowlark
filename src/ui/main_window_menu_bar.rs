use gtk::gio::Menu;
use gtk::PopoverMenuBar;

pub fn setup() -> PopoverMenuBar {
    let file_section_model = Menu::new();
    file_section_model.append(Some("Open"), None);

    let edit_section_model = Menu::new();

    let view_section_model = Menu::new();

    let help_section_model = Menu::new();
    help_section_model.append(Some("About"), Some("win.open-about-dialog"));

    let menu_bar_model = Menu::new();
    menu_bar_model.append_submenu(Some("File"), &file_section_model);
    menu_bar_model.append_submenu(Some("Edit"), &edit_section_model);
    menu_bar_model.append_submenu(Some("View"), &view_section_model);
    menu_bar_model.append_submenu(Some("Help"), &help_section_model);

    gtk::PopoverMenuBar::from_model(Some(&menu_bar_model))
}
