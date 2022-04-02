use vizia::*;

#[derive(Lens)]
pub struct IconData {
    is_hovering: bool,
}

impl Model for IconData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(icon_event) = event.message.downcast() {
            match icon_event {
                IconEvent::StartHover => {
                    self.is_hovering = true;
                    println!("Hover")
                }

                IconEvent::StopHover => {
                    self.is_hovering = false;
                    println!("UNHover")
                }
            }
        }
    }
}

pub struct Icon {
    is_hovering: bool,
}

impl Icon {
    pub fn new<'a>(cx: &'a mut Context, icon: &'a str) -> Handle<'a, Self> {
        Self { is_hovering: false }.build2(cx, |cx| {
            IconData { is_hovering: false }.build(cx);

            Label::new(cx, icon)
                // .width(Pixels(16.0))
                // .height(Pixels(16.0))
                // .font_size(16.0)
                .width(Pixels(150.0))
                .height(Pixels(150.0))
                .font_size(150.0)
                .font("meadowlark")
                .position_type(PositionType::SelfDirected)
                .z_order(15)
                .class("icon")
                .toggle_class("icon_hover", IconData::is_hovering);
        })
    }
}

pub enum IconEvent {
    StartHover,
    StopHover,
}

impl View for Icon {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                // Didn't get the hover to work properly
                WindowEvent::MouseEnter => cx.emit(IconEvent::StartHover),

                WindowEvent::MouseLeave => cx.emit(IconEvent::StopHover),

                _ => {}
            }
        }
    }
}
