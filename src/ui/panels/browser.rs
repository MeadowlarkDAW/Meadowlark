use vizia::prelude::*;

use crate::ui::ResizableStack;

pub fn browser(cx: &mut Context) {
    ResizableStack::new(cx, |_| {}).width(Pixels(160.0)).class("browser");
}
