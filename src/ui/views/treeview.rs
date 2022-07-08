use std::path::PathBuf;

use vizia::prelude::*;

// Represents an item which can have a list of sub-items.
#[derive(Lens)]
pub struct Directory {
    pub is_open: bool,
    pub path: Option<PathBuf>,
}

pub enum DirectoryEvent {
    // Toggles the `is_open` state of the view, causing the contents to be hidden or shown.
    ToggleOpen,
    Select,
}

impl Directory {
    pub fn new<'a>(
        cx: &'a mut Context,
        path: &Option<PathBuf>,
        header: impl FnOnce(&mut Context),
        content: impl FnOnce(&mut Context),
    ) -> Handle<'a, Self> {
        Self { is_open: true, path: path.clone() }
            .build(cx, |cx| {
                // Header
                // Pressing on the header will toggle the display mode of the contents.
                HStack::new(cx, header).height(Pixels(20.0)).on_press(|cx| {
                    cx.emit(DirectoryEvent::ToggleOpen);
                });
                // Content
                // The display of the contents is bound to the `is_open` state.
                VStack::new(cx, content).height(Auto).display(Directory::is_open);
            })
            .height(Auto)
    }
}

impl View for Directory {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|directory_event, meta| match directory_event {
            // Toggle the `is_open` state when the view receives the `ToggleOpen` message.
            DirectoryEvent::ToggleOpen => {
                self.is_open ^= true;
                meta.consume();
                println!("{:?}", self.path);
            }

            DirectoryEvent::Select => {
                println!("{:?}", self.path);
                meta.consume();
            }
        });
    }
}
