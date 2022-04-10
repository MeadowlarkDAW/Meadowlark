use vizia::*;

#[derive(Lens)]
pub struct ResizableStack {
    is_dragging: bool,
    on_drag: Option<Box<dyn Fn(&mut Context, f32)>>,
}

impl ResizableStack {
    pub fn new<F>(cx: &mut Context, content: F) -> Handle<Self>
    where
        F: FnOnce(&mut Context),
    {
        Self { is_dragging: false, on_drag: None }.build(cx, |cx| {
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
        if let Some(resizable_stack_event) = event.message.downcast() {
            match resizable_stack_event {
                ResizableStackEvent::StartDrag => {
                    self.is_dragging = true;
                    cx.capture();
                    // Prevent propagation in case the resizable stack is within another resizable stack
                    event.consume();
                }

                ResizableStackEvent::StopDrag => {
                    self.is_dragging = false;
                    cx.release();
                    event.consume()
                }
            }
        }

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseMove(x, _) => {
                    if self.is_dragging {
                        let posx = cx.cache.get_posx(cx.current);
                        let new_width = *x - posx;
                        cx.current.set_width(cx, Pixels(new_width));

                        if let Some(callback) = &self.on_drag {
                            (callback)(cx, new_width);
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

pub trait ResizableStackHandle {
    fn on_drag(self, callback: impl Fn(&mut Context, f32) + 'static) -> Self;
}

impl<'a> ResizableStackHandle for Handle<'a, ResizableStack> {
    fn on_drag(self, callback: impl Fn(&mut Context, f32) + 'static) -> Self {
        if let Some(resizable_stack) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|view_handler| view_handler.downcast_mut::<ResizableStack>())
        {
            resizable_stack.on_drag = Some(Box::new(callback));
        }

        self
    }
}
