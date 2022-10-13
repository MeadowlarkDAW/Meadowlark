mod imp;

use glib::Object;
use gtk::glib;

pub use imp::BrowserPanelItemType;

glib::wrapper! {
    pub struct BrowserPanelListItem(ObjectSubclass<imp::BrowserPanelListItem>);
}

impl BrowserPanelListItem {
    pub fn new(index: u32, item_type: BrowserPanelItemType, name: String) -> Self {
        Object::new(&[("index", &index), ("item-type", &item_type.to_u8()), ("name", &name)])
            .unwrap()
    }
}
