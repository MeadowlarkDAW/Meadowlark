mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct BrowserPanelListItem(ObjectSubclass<imp::BrowserPanelListItem>);
}

impl BrowserPanelListItem {
    pub fn new(number: i32) -> Self {
        Object::new(&[("number", &number)]).expect("Failed to create `IntegerObject`.")
    }
}
