use vizia::prelude::*;

// Represents an item which can have a list of sub-items
#[derive(Lens)]
pub struct Directory {
    pub is_open: bool,
}

enum DirectoryEvent {
    ToggleOpen,
}

impl Directory {
    pub fn new(
        cx: &mut Context,
        header: impl FnOnce(&mut Context),
        content: impl FnOnce(&mut Context),
    ) -> Handle<Self> {
        Self { is_open: true }
            .build(cx, |cx| {
                // Header
                HStack::new(cx, header)
                    .height(Pixels(20.0))
                    //.background_color(Color::red())
                    .on_press(|cx| {
                        cx.emit(DirectoryEvent::ToggleOpen);
                    });
                // Content
                VStack::new(cx, content)
                    .height(Auto)
                    //.background_color(Color::blue())
                    .display(Directory::is_open);
            })
            .height(Auto)
    }
}

impl View for Directory {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|directory_event, meta| match directory_event {
            DirectoryEvent::ToggleOpen => {
                self.is_open ^= true;
                meta.consume();
            }
        });
    }
}
