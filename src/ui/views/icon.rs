use vizia::*;

#[derive(Lens)]
pub struct IconData {}

impl Model for IconData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
}

pub struct Icon {}

impl Icon {
    pub fn new<'a>(cx: &'a mut Context, icon: &'a str, size: f32) -> Handle<'a, Self> {
        Self {}.build2(cx, |cx| {
            IconData {}.build(cx);

            Label::new(cx, icon)
                .width(Pixels(size))
                .height(Pixels(size))
                .font_size(size)
                .font("meadowlark")
                .position_type(PositionType::SelfDirected)
                .z_order(15)
                .class("icon");
        })
    }
}

pub enum IconEvent {}

impl View for Icon {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
}
