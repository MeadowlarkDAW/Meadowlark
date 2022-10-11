mod imp;

use glib::Object;
use gtk::glib;

pub use imp::BrowserPanelItemType;

glib::wrapper! {
    pub struct BrowserPanelListItem(ObjectSubclass<imp::BrowserPanelListItem>);
}

impl BrowserPanelListItem {
    pub fn new(id: u64, item_type: BrowserPanelItemType, name: String) -> Self {
        Object::new(&[("id", &id), ("item-type", &item_type.to_u8()), ("name", &name)]).unwrap()
    }
}
