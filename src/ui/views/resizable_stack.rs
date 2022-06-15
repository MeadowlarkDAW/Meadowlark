use vizia::prelude::*;

#[derive(Lens)]
pub struct ResizableStack {
    is_dragging: bool,
}

impl ResizableStack {
    pub fn new<F>(cx: &mut Context, content: F) -> Handle<Self>
    where
        F: FnOnce(&mut Context),
    {
        Self { is_dragging: false }.build(cx, |cx| {
            Element::new(cx)
                .width(Pixels(6.0))
                .left(Stretch(1.0))
                .right(Pixels(-3.0))
                .position_type(PositionType::SelfDirected)
                .z_order(10)
                .class("resize_handle")
                .toggle_class("drag_handle", ResizableStack::is_dragging)
                .cursor(CursorIcon::EwResize)
                .on_press(|cx| cx.emit(ResizableStackEvent::StartDrag));

            (content)(cx);
        })
    }
}

pub enum ResizableStackEvent {
    StartDrag,
    StopDrag,
}

impl View for ResizableStack {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|resizable_stack_event, event| match resizable_stack_event {
            ResizableStackEvent::StartDrag => {
                self.is_dragging = true;
                cx.capture();
                cx.lock_cursor_icon();
                // Prevent propagation in case the resizable stack is within another resizable stack
                event.consume();
            }

            ResizableStackEvent::StopDrag => {
                self.is_dragging = false;
                cx.release();
                cx.unlock_cursor_icon();
                event.consume()
            }
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::MouseMove(x, _) => {
                if self.is_dragging {
                    let current = cx.current();
                    let posx = cx.cache().get_posx(current);
                    let dpi = cx.style().dpi_factor as f32;
                    let new_width = (*x - posx) / dpi;
                    cx.style().width.insert(current, Pixels(new_width));
                    cx.style().needs_restyle = true;
                    cx.style().needs_relayout = true;
                    cx.style().needs_redraw = true;
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(ResizableStackEvent::StopDrag);
            }

            _ => {}
        });
    }
}
