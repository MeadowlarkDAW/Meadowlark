use vizia::prelude::*;

// A view with a header and contents arranged into a vertical column.
pub struct Panel {}

impl Panel {
    pub fn new(
        cx: &mut Context,
        header: impl FnOnce(&mut Context),
        content: impl FnOnce(&mut Context),
    ) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            // Header
            HStack::new(cx, header).class("header");

            // Contents
            VStack::new(cx, content).class("level3");
        })
    }
}

impl View for Panel {
    fn element(&self) -> Option<&'static str> {
        Some("panel")
    }
}
