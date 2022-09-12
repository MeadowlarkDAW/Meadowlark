use crate::ui::{
    state::{LaneState, LaneStates, TimelineGridState},
    PanelEvent, PanelState, ResizableStack, UiData, UiEvent, UiState,
};
use vizia::prelude::*;

pub const DEFAULT_LANE_HEIGHT_PX: f32 = 100.0;

#[derive(Lens)]
pub struct LaneHeader {
    index: usize,
    is_dragging: bool,
    on_drag: Box<dyn Fn(&mut EventContext, f32)>,
}

impl LaneHeader {
    pub fn new<L>(cx: &mut Context, item: L, index: usize) -> Handle<Self>
    where
        L: Lens<Target = LaneState> + Copy,
    {
        Self {
            index,
            is_dragging: false,
            on_drag: Box::new(move |cx, height| {
                cx.emit(UiEvent::SetSelectedLaneHeight(index, height))
            }),
        }
        .build(cx, |cx| {
            // Resize marker
            Element::new(cx)
                .height(Pixels(6.0))
                .top(Stretch(1.0))
                .bottom(Pixels(-3.0))
                //.background_color(Color::red())
                .position_type(PositionType::SelfDirected)
                .z_order(10)
                .class("resize_handle")
                .toggle_class("drag_handle", LaneHeader::is_dragging)
                .cursor(CursorIcon::NsResize)
                .on_press(|cx| cx.emit(LaneHeaderEvent::StartDrag));

            // Lane name
            Label::new(
                cx,
                item.then(LaneState::name).map(move |x| match x {
                    Some(lane) => (*lane).clone(),
                    None => format!("lane {}", index),
                }),
            )
            .class("lane_name");

            // Lane color bar
            Element::new(cx)
                .bind(item.then(LaneState::color), move |handle, color| {
                    handle.bind(item.then(LaneState::disabled), move |handle, disabled| {
                        if !disabled.get(handle.cx) {
                            handle.background_color(color.clone().map(|x| match x {
                                Some(color) => (*color).clone().into(),
                                None => Color::from("#888888"),
                            }));
                        } else {
                            handle.background_color(Color::from("#444444"));
                        }
                    });
                })
                .class("lane_bar");
        })
        .layout_type(LayoutType::Row)
        //.on_press(move |cx| {
        //cx.emit(UiEvent::SelectLane(index));
        //cx.focus();
        //})
        .bind(item.then(LaneState::height), move |handle, height| {
            let factor = match height.get(handle.cx) {
                Some(height) => height as f32,
                None => 1.0,
            };
            handle.bind(
                UiData::state
                    .then(UiState::timeline_grid.then(TimelineGridState::vertical_zoom_level)),
                move |handle, zoom_y| {
                    let zoom_y = zoom_y.get(handle.cx) as f32;
                    handle.height(Pixels(factor * DEFAULT_LANE_HEIGHT_PX * zoom_y));
                },
            );
        })
        .class("lane_header")
        .toggle_class("selected", item.then(LaneState::selected))
        .toggle_class("disabled", item.then(LaneState::disabled))
    }
}

pub(crate) enum LaneHeaderEvent {
    StartDrag,
    StopDrag,
}

impl View for LaneHeader {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|resizable_stack_event, event| match resizable_stack_event {
            LaneHeaderEvent::StartDrag => {
                self.is_dragging = true;
                println!("Set is dragging: {}", cx.current());
                cx.capture();
                cx.lock_cursor_icon();
                // Prevent propagation in case the resizable stack is within another resizable stack
                event.consume();
            }

            LaneHeaderEvent::StopDrag => {
                self.is_dragging = false;
                cx.release();
                cx.unlock_cursor_icon();
                event.consume()
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                if meta.target == cx.current() {
                    // TODO: Move this to `on_press` action modifier once vizia#180 is merged.
                    cx.emit(UiEvent::SelectLane(self.index));
                    cx.focus();
                }
            }

            WindowEvent::MouseMove(_, y) => {
                if self.is_dragging {
                    let current = cx.current();
                    let posy = cx.cache.get_posy(current);
                    let dpi = cx.scale_factor();
                    let new_height = (*y - posy) / dpi;
                    (self.on_drag)(cx, new_height / DEFAULT_LANE_HEIGHT_PX);
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(LaneHeaderEvent::StopDrag);
            }

            _ => {}
        });
    }
}

pub fn lane_header(cx: &mut Context) {
    ResizableStack::new(
        cx,
        UiData::state.then(UiState::panels.then(PanelState::lane_header_width)),
        |cx, width| {
            cx.emit(PanelEvent::SetLaneHeaderWidth(width));
        },
        |cx| {
            List::new(
                cx,
                UiData::state.then(
                    UiState::timeline_grid
                        .then(TimelineGridState::lane_states.then(LaneStates::lanes)),
                ),
                move |cx, index, item| {
                    LaneHeader::new(cx, item, index);
                },
            )
            .class("lane_headers");
        },
    );
}

pub fn lane_content(cx: &mut Context) {
    // TODO: Implement
}
