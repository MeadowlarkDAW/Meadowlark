use std::path::{Path, PathBuf};

use super::UiEvent;
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub root_file: File,
    pub selected: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    ViewAll,
    SetRootPath(PathBuf),
    SetSelected(PathBuf),
    SelectNext,
    SelectPrev,
    ToggleOpen,
    PlaySelected,
    StopSelected,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct File {
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub children: Vec<File>,
    pub is_open: bool,
}

impl Default for File {
    fn default() -> Self {
        Self { name: String::new(), file_path: None, children: Vec::new(), is_open: true }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            root_file: File {
                name: String::from("root"),
                file_path: Some(PathBuf::from("assets/test_files")),
                children: vec![],
                is_open: true,
            },
            selected: Some(PathBuf::from("assets/test_files")),
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            // Temp: Load the assets directory for the treeview
            BrowserEvent::ViewAll => {
                if let Some(root) = visit_dirs(&Path::new("assets/test_files")) {
                    self.root_file = root;
                }
            }

            BrowserEvent::SetRootPath(path) => {
                if let Some(root) = visit_dirs(path.as_path()) {
                    self.root_file = root;
                }
            }

            // Play the selected file
            BrowserEvent::PlaySelected => {
                if let Some(path) = &self.selected {
                    if path.is_file() {
                        cx.emit(UiEvent::BrowserFileClicked(path.clone()));
                    }
                }
            }

            BrowserEvent::StopSelected => {
                cx.emit(UiEvent::BrowserFileStop());
            }

            BrowserEvent::ToggleOpen => {
                //println!("Toggle Open: {:?}", path);
                if let Some(path) = &self.selected {
                    toggle_open(&mut self.root_file, path);
                }
            }

            // Set the selected directory item by path
            BrowserEvent::SetSelected(path) => {
                self.selected = Some(path.clone());
            }

            // Move selection the next directory item
            BrowserEvent::SelectNext => {
                let next = recursive_next(&self.root_file, None, self.selected.clone());
                match next {
                    RetItem::Found(path) => self.selected = path,
                    _ => {}
                }
            }

            // Move selection the previous directory item
            BrowserEvent::SelectPrev => {
                let next = recursive_prev(&self.root_file, None, self.selected.clone());
                match next {
                    RetItem::Found(path) => self.selected = path,
                    _ => {}
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(Option<PathBuf>),
    NotFound(Option<&'a File>),
}

fn toggle_open(root: &mut File, path: &PathBuf) {
    if root.file_path == Some(path.clone()) {
        root.is_open ^= true;
    } else {
        for child in root.children.iter_mut() {
            toggle_open(child, path);
        }
    }
}

// Returns the next directory item after `dir` by recursing down the hierarchy
fn recursive_next<'a>(
    root: &'a File,
    mut prev: Option<&'a File>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if let Some(prev) = prev {
        if prev.file_path == dir {
            return RetItem::Found(root.file_path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_next(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Returns the previous directory item before `dir` by recursing down the hierarchy
fn recursive_prev<'a>(
    root: &'a File,
    mut prev: Option<&'a File>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if root.file_path == dir {
        if let Some(prev) = prev {
            return RetItem::Found(prev.file_path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_prev(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Recursively build directory tree from root path
fn visit_dirs(dir: &Path) -> Option<File> {
    let name = format!("{}", dir.file_name()?.to_str()?);
    let mut children = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                children.push(visit_dirs(&path)?);
            } else {
                children.push(File {
                    name: format!("{}", entry.path().file_name()?.to_str()?),
                    file_path: Some(entry.path()),
                    children: vec![],
                    is_open: true,
                })
            }
        }
    }

    // Sort by alphabetical
    children.sort_by(|a, b| a.name.cmp(&b.name));
    // Sort by directory vs file
    children.sort_by(|a, b| {
        let a_is_dir: bool = a.children.is_empty();
        let b_is_dir: bool = b.children.is_empty();
        a_is_dir.cmp(&b_is_dir)
    });

    Some(File { name, file_path: Some(PathBuf::from(dir)), children, is_open: true })
}

// Return the path of a file directory
fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
