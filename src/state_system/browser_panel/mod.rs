use fnv::FnvHashMap;
use gtk::gio::{ListModel, ListStore};
use gtk::glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};
use gtk::{prelude::*, TreeListModel, TreeModel};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use walkdir::WalkDir;

mod list_item;
mod tree_list_item;
pub use list_item::{BrowserPanelItemType, BrowserPanelListItem};
pub use tree_list_item::BrowserPanelTreeListItem;

use super::app_message::AppMessage;

static MAX_FOLDER_SCAN_DEPTH: usize = 12;

// TODO: Store file and folder paths in a more efficient way. Currently
// the full path is stored in every folder & file entry.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserCategory {
    Samples,
}

pub struct BrowserPanelState {
    pub shown: bool,

    pub selected_category: BrowserCategory,

    pub samples_folder_tree_model: FolderTreeModel,
    pub samples_folder_tree_selected_id: Option<u64>,
    pub samples_folder_tree_scanning: bool,
    next_entry_id: u64,

    pub sample_directories: Vec<PathBuf>,

    /*
    pub top_panel_list_items: Vec<BrowserPanelListItem>,
    pub top_panel_list_model: ListStore,
    */
    pub file_index_to_path: Vec<PathBuf>,
    pub file_list_pre_model: Vec<BrowserPanelItemEntry>,
    pub file_list_model: Option<ListStore>,
    pub latest_file_scan_id: Arc<AtomicU64>,
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
            samples_folder_tree_scanning: false,
            sample_directories: vec![test_samples_directory],
            next_entry_id: 0,
            file_index_to_path: Vec::new(),
            file_list_pre_model: Vec::new(),
            file_list_model: None,
            latest_file_scan_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_browser_folder(
        &mut self,
        id: u64,
        app_msg_tx: &gtk::glib::Sender<AppMessage>,
    ) -> bool {
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
                        self.refresh_file_list(path, app_msg_tx);
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    pub fn refresh_folder_tree(&mut self, app_msg_tx: &gtk::glib::Sender<AppMessage>) -> bool {
        match self.selected_category {
            BrowserCategory::Samples => {
                if self.samples_folder_tree_scanning {
                    // A scan operation is already taking place.
                    return false;
                }

                self.samples_folder_tree_scanning = true;

                self.samples_folder_tree_model.clear();
                self.samples_folder_tree_selected_id = None;

                let app_msg_tx = app_msg_tx.clone();
                let mut next_entry_id = self.next_entry_id;
                let category = self.selected_category;
                let mut folder_tree_model = FolderTreeModel::new();
                // Attempt to reuse the allocated memory from the last scan.
                std::mem::swap(&mut self.samples_folder_tree_model, &mut folder_tree_model);

                let directories = self.sample_directories.clone();

                std::thread::spawn(move || {
                    for directory in directories.iter() {
                        let id = next_entry_id;
                        next_entry_id += 1;
                        folder_tree_model.entry_id_to_path.insert(id, directory.clone());

                        let name = directory
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| "<error>".into());

                        let children = read_subdirectories(
                            directory,
                            &mut next_entry_id,
                            &mut folder_tree_model.entry_id_to_path,
                            0,
                        );

                        folder_tree_model.entries.push(FolderTreeEntry { id, name, children });
                    }

                    app_msg_tx
                        .send(AppMessage::BrowserPanelFolderTreeRefreshed {
                            category,
                            folder_tree_model,
                            next_entry_id,
                        })
                        .unwrap();
                });

                true
            }
        }
    }

    fn refresh_file_list(
        &mut self,
        directory: PathBuf,
        app_msg_tx: &gtk::glib::Sender<AppMessage>,
    ) {
        self.file_index_to_path.clear();
        self.file_list_pre_model.clear();
        self.file_list_model = None;

        let app_msg_tx = app_msg_tx.clone();
        let latest_file_scan_id = Arc::clone(&self.latest_file_scan_id);
        let mut file_index_to_path = Vec::new();
        let mut file_list_pre_model: Vec<BrowserPanelItemEntry> = Vec::new();
        // Attempt to reuse the allocated memory from the last scan.
        std::mem::swap(&mut self.file_index_to_path, &mut file_index_to_path);
        std::mem::swap(&mut self.file_list_pre_model, &mut file_list_pre_model);

        let file_scan_id = self.latest_file_scan_id.load(Ordering::SeqCst) + 1;
        self.latest_file_scan_id.store(file_scan_id, Ordering::SeqCst);
        std::thread::spawn(move || {
            let mut do_send_result = true;

            let walker = walkdir::WalkDir::new(directory)
                .max_depth(MAX_FOLDER_SCAN_DEPTH)
                .follow_links(false);

            for entry in walker {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        if let Some(extension) = entry.path().extension() {
                            if let Some(extension) = extension.to_str() {
                                // TODO: More file types
                                let item_type = match extension {
                                    "wav" | "flac" | "ogg" | "mp3" => {
                                        Some(BrowserPanelItemType::Audio)
                                    }
                                    _ => None,
                                };

                                if let Some(item_type) = item_type {
                                    file_list_pre_model.push(BrowserPanelItemEntry {
                                        item_type,
                                        name: entry.file_name().to_string_lossy().to_string(),
                                        path: entry.path().to_path_buf(),
                                    });
                                }
                            }
                        }
                    }
                }

                // If a new scan was started before this one has finished, then
                // cancel this scan.
                if latest_file_scan_id.load(Ordering::Relaxed) > file_scan_id {
                    do_send_result = false;
                    break;
                }
            }

            if !do_send_result {
                return;
            }

            // Sort items alphanumerically.
            file_list_pre_model.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

            // If a new scan was started before this one has finished, then
            // cancel this scan.
            if latest_file_scan_id.load(Ordering::SeqCst) > file_scan_id {
                return;
            }

            app_msg_tx
                .send(AppMessage::BrowserPanelFileListRefreshed {
                    file_scan_id,
                    file_list_pre_model,
                })
                .unwrap();
        });
    }

    pub fn on_folder_tree_refreshed(
        &mut self,
        category: BrowserCategory,
        folder_tree_model: FolderTreeModel,
        next_entry_id: u64,
    ) -> Option<&FolderTreeModel> {
        self.next_entry_id = self.next_entry_id.max(next_entry_id);

        match category {
            BrowserCategory::Samples => {
                self.samples_folder_tree_scanning = false;
                self.samples_folder_tree_model = folder_tree_model;

                Some(&self.samples_folder_tree_model)
            }
        }
    }

    pub fn on_file_list_refreshed(
        &mut self,
        file_scan_id: u64,
        mut file_list_pre_model: Vec<BrowserPanelItemEntry>,
    ) -> Option<&ListStore> {
        // If this is the result of an outdated scan, then ignore it.
        if file_scan_id < self.latest_file_scan_id.load(Ordering::SeqCst) {
            return None;
        }

        let new_model = ListStore::new(BrowserPanelListItem::static_type());
        self.file_index_to_path.clear();
        for (i, item) in file_list_pre_model.drain(..).enumerate() {
            self.file_index_to_path.push(item.path);
            new_model.append(&BrowserPanelListItem::new(i as u32, item.item_type, item.name));
        }
        self.file_list_model = Some(new_model);

        self.file_list_pre_model = file_list_pre_model;

        self.file_list_model.as_ref()
    }

    pub fn on_browser_item_selected(&mut self, index: u32) -> Option<PathBuf> {
        self.file_index_to_path.get(index as usize).map(|p| p.clone())
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

pub struct BrowserPanelItemEntry {
    item_type: BrowserPanelItemType,
    name: String,
    path: PathBuf,
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
