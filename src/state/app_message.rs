use fnv::FnvHashMap;
use gtk::ListStore;
use std::path::PathBuf;

use super::browser_panel::{BrowserCategory, BrowserPanelItemEntry, FolderTreeModel};

pub enum AppMessage {
    BrowserPanelFolderTreeRefreshed {
        category: BrowserCategory,
        folder_tree_model: FolderTreeModel,
        next_entry_id: u64,
    },
    BrowserPanelFileListRefreshed {
        file_scan_id: u64,
        file_list_pre_model: Vec<BrowserPanelItemEntry>,
        file_id_to_path: FnvHashMap<u64, PathBuf>,
        next_entry_id: u64,
    },
}
