use vizia::prelude::*;

// A view which can be resized by clicking and dragging from the right edge of the view.
#[derive(Lens)]
pub struct ResizableHStackDragR {
    // State which tracks whether the edge of the view is being dragged.
    is_dragging: bool,
    // Callback which is triggered when the view is being dragged.
    on_drag: Box<dyn Fn(&mut EventContext, f32)>,
}

impl ResizableHStackDragR {
    pub fn new<F>(
        cx: &mut Context,
        width: impl Lens<Target = f32>,
        on_drag: impl Fn(&mut EventContext, f32) + 'static,
        content: F,
    ) -> Handle<Self>
    where
        F: FnOnce(&mut Context),
    {
        Self { is_dragging: false, on_drag: Box::new(on_drag) }
            .build(cx, |cx| {
                Element::new(cx)
                    .width(Pixels(6.0))
                    .left(Stretch(1.0))
                    .right(Pixels(-3.0))
                    .position_type(PositionType::SelfDirected)
                    .z_order(10)
                    .class("resize_handle")
                    .toggle_class("drag_handle", ResizableHStackDragR::is_dragging)
                    .cursor(CursorIcon::EwResize)
                    .on_mouse_down(|cx, button| {
                        if button == MouseButton::Left {
                            cx.emit(ResizableHStackDragREvent::StartDrag);
                        }
                    });

                (content)(cx);
            })
            .width(width.map(|w| Units::Pixels(*w)))
    }
}

pub enum ResizableHStackDragREvent {
    StartDrag,
    StopDrag,
}

impl View for ResizableHStackDragR {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|resizable_stack_event, event| match resizable_stack_event {
            ResizableHStackDragREvent::StartDrag => {
                self.is_dragging = true;
                cx.capture();
                cx.lock_cursor_icon();
                // Prevent propagation in case the resizable stack is within another resizable stack
                event.consume();
            }

            ResizableHStackDragREvent::StopDrag => {
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
                    let posx = cx.cache.get_posx(current);
                    let scale_factor = cx.style.dpi_factor as f32;
                    let new_width = (*x - posx) / scale_factor;
                    (self.on_drag)(cx, new_width);
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(ResizableHStackDragREvent::StopDrag);
            }

            _ => {}
        });
    }
}
