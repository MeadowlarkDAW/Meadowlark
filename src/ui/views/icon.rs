use vizia::*;

use crate::ui::icons::IconType;

#[derive(Lens)]
pub struct IconData {}

impl Model for IconData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
}

pub struct Icon {}

impl Icon {
    pub fn new<'a>(cx: &'a mut Context, icon: IconType) -> Handle<'a, Self> {
        Self {}.build2(cx, |cx| {

            let size = 24.0 * 4.0;
            let icon_str: &str = icon.into();

            IconData {}.build(cx);

            HStack::new(cx, |cx| {
                Label::new(cx, icon_str)
                    .width(Pixels(size * 0.666))
                    .height(Pixels(size * 0.666))
                    .font_size(size * 0.666)
                    .font("meadowlark")
                    .position_type(PositionType::SelfDirected)
                    .z_order(15)
                    .class("icon-svg");
            })
            .width(Pixels(size))
            .height(Pixels(size))
            .class("icon");
        })
    }
}

pub enum IconEvent {}

impl View for Icon {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
}
