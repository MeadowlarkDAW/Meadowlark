use fnv::FnvHashMap;
use gtk::gio::{ListModel, ListStore};
use gtk::{prelude::*, TreeListModel, TreeModel};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod list_item;
mod tree_list_item;
pub use list_item::{BrowserPanelItemType, BrowserPanelListItem};
pub use tree_list_item::BrowserPanelTreeListItem;

static MAX_FOLDER_SCAN_DEPTH: usize = 12;

// TODO: Store file and folder paths in a more efficient way. Currently
// the full path of each folder/file is stored in each entry.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserCategory {
    Samples,
}

pub struct BrowserPanelState {
    pub shown: bool,

    pub selected_category: BrowserCategory,

    pub samples_folder_tree_model: FolderTreeModel,
    pub samples_folder_tree_selected_id: Option<u64>,
    next_entry_id: u64,

    pub sample_directories: Vec<PathBuf>,

    /*
    pub top_panel_list_items: Vec<BrowserPanelListItem>,
    pub top_panel_list_model: ListStore,
    */
    pub file_id_to_path: FnvHashMap<u64, PathBuf>,
    pub file_list_model: Option<ListStore>,
}

impl BrowserPanelState {
    pub fn new() -> Self {
        let mut test_samples_directory = std::env::current_dir().unwrap();
        test_samples_directory.push("assets");
        test_samples_directory.push("test_files");

        Self {
            shown: true,
            selected_category: BrowserCategory::Samples,
            samples_folder_tree_model: FolderTreeModel::new(),
            samples_folder_tree_selected_id: None,
            sample_directories: vec![test_samples_directory],
            next_entry_id: 0,
            file_id_to_path: FnvHashMap::default(),
            file_list_model: None,
        }
    }

    pub fn set_browser_folder(&mut self, id: u64) -> bool {
        match self.selected_category {
            BrowserCategory::Samples => {
                let new_browser_folder_selected =
                    if let Some(old_id) = self.samples_folder_tree_selected_id {
                        old_id != id
                    } else {
                        true
                    };

                if new_browser_folder_selected {
                    if let Some(path) =
                        self.samples_folder_tree_model.entry_id_to_path.get(&id).map(|e| e.clone())
                    {
                        self.refresh_file_list(path);
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    pub fn refresh_folder_tree(&mut self) -> Option<&FolderTreeModel> {
        match self.selected_category {
            BrowserCategory::Samples => {
                self.samples_folder_tree_model.clear();
                self.samples_folder_tree_selected_id = None;

                for directory in self.sample_directories.iter() {
                    let id = self.next_entry_id;
                    self.next_entry_id += 1;
                    self.samples_folder_tree_model.entry_id_to_path.insert(id, directory.clone());

                    let name = directory
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| "<error>".into());

                    let children = read_subdirectories(
                        directory,
                        &mut self.next_entry_id,
                        &mut self.samples_folder_tree_model.entry_id_to_path,
                        0,
                    );

                    self.samples_folder_tree_model.entries.push(FolderTreeEntry {
                        id,
                        name,
                        children,
                    });

                    return Some(&self.samples_folder_tree_model);
                }
            }
        }

        None
    }

    fn refresh_file_list(&mut self, directory: PathBuf) {
        self.file_id_to_path.clear();

        struct ItemEntry {
            id: u64,
            item_type: BrowserPanelItemType,
            name: String,
        }

        let mut items: Vec<ItemEntry> = Vec::new();

        let walker =
            walkdir::WalkDir::new(directory).max_depth(MAX_FOLDER_SCAN_DEPTH).follow_links(false);

        for entry in walker {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Some(extension) = entry.path().extension() {
                        if let Some(extension) = extension.to_str() {
                            // TODO: More file types
                            let item_type = match extension {
                                "wav" | "flac" | "ogg" | "mp3" => Some(BrowserPanelItemType::Audio),
                                _ => None,
                            };

                            if let Some(item_type) = item_type {
                                let id = self.next_entry_id;
                                self.next_entry_id += 1;

                                let path = entry.path().to_path_buf();
                                self.file_id_to_path.insert(id, path);

                                items.push(ItemEntry {
                                    id,
                                    item_type,
                                    name: entry.file_name().to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort items alphanumerically.
        items.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

        let new_list_store = ListStore::new(BrowserPanelListItem::static_type());
        for item in items.drain(..) {
            new_list_store.append(&BrowserPanelListItem::new(item.id, item.item_type, item.name));
        }

        self.file_list_model = Some(new_list_store);
    }
}

fn read_subdirectories(
    directory_path: &PathBuf,
    next_entry_id: &mut u64,
    entry_id_to_path: &mut FnvHashMap<u64, PathBuf>,
    current_depth: usize,
) -> Vec<FolderTreeEntry> {
    assert!(directory_path.is_dir());

    if current_depth >= MAX_FOLDER_SCAN_DEPTH {
        log::warn!("Reached maximum depth of {} while reading directories", MAX_FOLDER_SCAN_DEPTH);
        return Vec::new();
    }

    let read_result = match std::fs::read_dir(directory_path.as_path()) {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to read contents of directory {:?}: {}", directory_path, e);
            return Vec::new();
        }
    };

    let mut entries: Vec<FolderTreeEntry> = Vec::new();
    for entry in read_result {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                continue;
            }
        };

        if let Ok(file_type) = &entry.file_type() {
            if file_type.is_dir() {
                let id = *next_entry_id;
                *next_entry_id += 1;

                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "(error)".into());

                let children =
                    read_subdirectories(&path, next_entry_id, entry_id_to_path, current_depth + 1);

                entry_id_to_path.insert(id, path);

                entries.push(FolderTreeEntry { id, name, children })
            }
        }
    }

    entries
}

pub struct FolderTreeModel {
    pub entries: Vec<FolderTreeEntry>,
    entry_id_to_path: FnvHashMap<u64, PathBuf>,
}

impl FolderTreeModel {
    pub fn new() -> Self {
        Self { entries: Vec::new(), entry_id_to_path: FnvHashMap::default() }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.entry_id_to_path.clear();
    }
}

pub struct FolderTreeEntry {
    pub id: u64,
    pub name: String,
    pub children: Vec<FolderTreeEntry>,
}
