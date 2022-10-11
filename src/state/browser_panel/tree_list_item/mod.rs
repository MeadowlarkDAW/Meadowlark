mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct BrowserPanelTreeListItem(ObjectSubclass<imp::BrowserPanelTreeListItem>);
}

impl BrowserPanelTreeListItem {
    pub fn new(number: i32, has_children: bool) -> Self {
        Object::new(&[("number", &number), ("has_children", &has_children)]).unwrap()
    }
}
