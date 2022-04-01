use vizia::*;

use crate::ui::ResizableStack;

pub fn browser(cx: &mut Context) {
    //VStack::new(cx, |cx| {}).text("Browser/Properties").class("browser");
    ResizableStack::new(cx).width(Pixels(160.0)).text("Browser/Properties").class("browser");
}
