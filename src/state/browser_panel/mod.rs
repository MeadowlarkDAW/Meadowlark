use gtk::gio::ListStore;
use gtk::prelude::*;

mod list_item;
pub use list_item::BrowserPanelListItem;

pub struct BrowserPanelState {
    pub shown: bool,

    pub top_panel_list_items: Vec<BrowserPanelListItem>,
    pub top_panel_list_model: ListStore,

    pub bottom_panel_list_items: Vec<BrowserPanelListItem>,
    pub bottom_panel_list_model: ListStore,
}

impl BrowserPanelState {
    pub fn new() -> Self {
        let top_panel_list_items: Vec<BrowserPanelListItem> =
            (0..=100).into_iter().map(BrowserPanelListItem::new).collect();
        let top_panel_list_model = ListStore::new(BrowserPanelListItem::static_type());
        top_panel_list_model.extend_from_slice(&top_panel_list_items);

        let bottom_panel_list_items: Vec<BrowserPanelListItem> =
            (0..=100).into_iter().map(BrowserPanelListItem::new).collect();
        let bottom_panel_list_model = ListStore::new(BrowserPanelListItem::static_type());
        bottom_panel_list_model.extend_from_slice(&bottom_panel_list_items);

        Self {
            shown: true,
            top_panel_list_items,
            top_panel_list_model,
            bottom_panel_list_items,
            bottom_panel_list_model,
        }
    }
}
