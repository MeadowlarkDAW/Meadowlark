use super::browser_panel::{BrowserCategory, BrowserPanelItemEntry, FolderTreeModel};

pub enum AppMessage {
    PollEngineTimer,
    BrowserPanelFolderTreeRefreshed {
        category: BrowserCategory,
        folder_tree_model: FolderTreeModel,
        next_entry_id: u64,
    },
    BrowserPanelFileListRefreshed {
        file_scan_id: u64,
        file_list_pre_model: Vec<BrowserPanelItemEntry>,
    },
}
