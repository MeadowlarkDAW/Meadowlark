use vizia::*;

use crate::ui::icons::IconCode;

pub struct Icon {}

impl Icon {
    pub fn new<'a>(cx: &'a mut Context, icon: IconCode, size: f32) -> Handle<'a, Self> {
        Self {}.build2(cx, |cx| {
            let icon_str: &str = icon.into();

            Label::new(cx, icon_str)
                .width(Pixels(size))
                .height(Pixels(size))
                .font_size(size * 0.666)
                .font("meadowlark")
                .position_type(PositionType::SelfDirected)
                .z_order(15)
                .class("icon");
        })
    }
}

impl View for Icon {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
}
