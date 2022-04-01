use vizia::*;

#[derive(Lens)]
pub struct ResizableStackData {
    is_dragging: bool,
}

impl Model for ResizableStackData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(resizable_stack_event) = event.message.downcast() {
            match resizable_stack_event {
                ResizableStackEvent::StartDrag => {
                    self.is_dragging = true;
                    cx.capture();
                }

                ResizableStackEvent::StopDrag => {
                    self.is_dragging = false;
                    cx.release();
                }
            }
        }
    }
}

pub struct ResizableStack {
    is_dragging: bool,
}

impl ResizableStack {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { is_dragging: false }.build2(cx, |cx| {
            ResizableStackData { is_dragging: false }.build(cx);

            Element::new(cx)
                .width(Pixels(6.0))
                .left(Stretch(1.0))
                .right(Pixels(-3.0))
                .position_type(PositionType::SelfDirected)
                .z_order(10)
                .class("resize_handle")
                .toggle_class("drag_handle", ResizableStackData::is_dragging)
                .cursor(CursorIcon::EwResize)
                .on_press(|cx| cx.emit(ResizableStackEvent::StartDrag));
        })
    }
}

pub enum ResizableStackEvent {
    StartDrag,
    StopDrag,
}

impl View for ResizableStack {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseMove(x, _) => {
                    if let Some(data) = cx.data::<ResizableStackData>() {
                        if data.is_dragging {
                            let posx = cx.cache.get_posx(cx.current);
                            let new_width = *x - posx;
                            cx.current.set_width(cx, Pixels(new_width));
                        }
                    }
                }

                WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                    cx.emit(ResizableStackEvent::StopDrag);
                }

                _ => {}
            }
        }
    }
}
