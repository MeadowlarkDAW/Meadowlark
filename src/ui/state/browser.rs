use std::{fs::DirEntry, path::Path};

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct BrowserState {
    #[serde(skip)]
    pub root_file: File,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct File {
    pub name: String,
    pub children: Vec<File>,
}

impl Default for File {
    fn default() -> Self {
        Self { name: String::new(), children: Vec::new() }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            // root_file: File {
            //     name: String::from("Documents"),
            //     children: vec![File {
            //         name: String::from("Work"),
            //         children: vec![File { name: String::from("Revision"), children: vec![] }],
            //     }],
            // },
            root_file: File { name: String::from("root"), children: vec![] },
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::ViewAll => {
                // for entry in WalkDir::new("assets/test_files").into_iter().filter_map(|e| e.ok()) {
                //     println!("{} {}", entry.path().display(), entry.depth());
                // }
                //if let Some(file) = get_file("assets/test_files") {
                //     self.root_file = file;
                // }

                if let Some(root) = visit_dirs(&Path::new("assets/test_files")) {
                    self.root_file = root;
                }
            }
        });
    }
}

fn callback(dir_entry: &DirEntry) {
    println!("{:?}", dir_entry.file_name());
}

fn visit_dirs(dir: &Path) -> Option<File> {
    let name = format!("{}", dir.file_name()?.to_str()?);
    let mut children = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                // cb(&entry);
                children.push(visit_dirs(&path)?);
            } else {
                //cb(&entry);

                children.push(File {
                    name: format!("{}", entry.path().file_name()?.to_str()?),
                    children: vec![],
                })
            }
        }
    }

    Some(File { name, children })
}

// pub fn get_file(path: impl AsRef<Path>) -> std::io::Result<File> {
//     if let Some(name) = path.as_ref().file_name().and_then(|filename| filename.to_str()) {
//         return Ok(File {
//             name: name.to_owned(),
//             children: std::fs::read_dir(path)?
//                 .filter_map(|e| e.ok())
//                 .filter_map(|dir_entry| get_file(dir_entry.path()))
//                 .collect::<Vec<_>>(),
//         });
//     }

//     None
// }

pub enum BrowserEvent {
    ViewAll,
}
