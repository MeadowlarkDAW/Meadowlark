use vizia::prelude::*;

use crate::state_system::app_state::PaletteColor;
use crate::ui::generic_views::{Icon, IconCode};

static THRESHOLD_HEIGHT: f32 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum BoundTrackHeaderType {
    Audio,
    Synth,
    Master,
}

#[derive(Debug, Lens, Clone, Data)]
pub struct BoundTrackHeaderState {
    pub name: String,
    pub color: PaletteColor,
    pub height: f32,
    pub type_: BoundTrackHeaderType,
    pub selected: bool,
}

// TODO: Double-click to reset to default height.

pub struct TrackHeaderView<L: Lens> {
    lens: L,
    is_resize_dragging: bool,
    on_event: Box<dyn Fn(&mut EventContext, TrackHeaderEvent)>,
    is_master_track: bool,
}

impl<L> TrackHeaderView<L>
where
    L: Lens<Target = BoundTrackHeaderState>,
{
    pub fn new<'a>(
        cx: &'a mut Context,
        lens: L,
        is_master_track: bool,
        on_event: impl Fn(&mut EventContext, TrackHeaderEvent) + 'static,
    ) -> Handle<'a, Self> {
        Self {
            lens: lens.clone(),
            is_resize_dragging: false,
            on_event: Box::new(on_event),
            is_master_track,
        }
        .build(cx, move |cx| {
            Binding::new(cx, lens, move |cx, state| {
                let state = state.get(cx);

                let header_view = HStack::new(cx, |cx| {
                    if is_master_track {
                        // Resize the master track from the top.
                        Element::new(cx)
                            .height(Pixels(6.0))
                            .top(Pixels(-3.0))
                            .bottom(Stretch(1.0))
                            .width(Stretch(1.0))
                            .position_type(PositionType::SelfDirected)
                            .z_order(10)
                            .cursor(CursorIcon::NsResize)
                            .on_mouse_down(|cx, button| {
                                if button == MouseButton::Left {
                                    cx.emit(InternalTrackHeaderEvent::StartResizeDrag);
                                }
                            });
                    } else {
                        // Resize all other tracks from the bottom.
                        Element::new(cx)
                            .height(Pixels(6.0))
                            .top(Stretch(1.0))
                            .bottom(Pixels(-3.0))
                            .width(Stretch(1.0))
                            .position_type(PositionType::SelfDirected)
                            .z_order(10)
                            .cursor(CursorIcon::NsResize)
                            .on_mouse_down(|cx, button| {
                                if button == MouseButton::Left {
                                    cx.emit(InternalTrackHeaderEvent::StartResizeDrag);
                                }
                            });
                    }

                    HStack::new(cx, |cx| {
                        // Don't show the grip icon for the master track.
                        if !is_master_track {
                            Label::new(cx, IconCode::GripVertical)
                                .font_size(19.0)
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0))
                                .width(Pixels(20.0))
                                .height(Pixels(20.0))
                                .position_type(PositionType::SelfDirected)
                                .class("grip")
                                .font("meadowlark-icons");
                        }
                    })
                    .width(Pixels(20.0))
                    .background_color(state.color.into_color());

                    if state.height >= THRESHOLD_HEIGHT {
                        // Full view
                        VStack::new(cx, |cx| {
                            Label::new(cx, &state.name).class("name");

                            // TODO: Fix icon sizes,
                            let (icon, icon_size) = match state.type_ {
                                BoundTrackHeaderType::Master => (IconCode::MasterTrack, 20.0),
                                BoundTrackHeaderType::Audio => (IconCode::Soundwave, 20.0),
                                BoundTrackHeaderType::Synth => (IconCode::Piano, 16.0),
                            };

                            Icon::new(cx, icon, 21.0, icon_size)
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0));
                        })
                        .left(Pixels(2.0))
                        .child_space(Pixels(4.0));

                        VStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                HStack::new(cx, |cx| {
                                    Button::new(
                                        cx,
                                        |_| {},
                                        |cx| Icon::new(cx, IconCode::Automation, 18.0, 16.0),
                                    )
                                    .class("icon_btn");
                                })
                                .class("button_group")
                                .height(Pixels(20.0))
                                .width(Auto);

                                HStack::new(cx, |cx| {
                                    Button::new(
                                        cx,
                                        |_| {},
                                        |cx| Icon::new(cx, IconCode::Record, 18.0, 16.0),
                                    )
                                    .class("icon_btn");
                                })
                                .class("button_group")
                                .left(Pixels(5.0))
                                .height(Pixels(20.0))
                                .width(Auto);

                                HStack::new(cx, |cx| {
                                    Button::new(
                                        cx,
                                        |_| {},
                                        |cx| Label::new(cx, "S").bottom(Pixels(3.0)),
                                    )
                                    .child_space(Stretch(1.0))
                                    .width(Pixels(20.0))
                                    .height(Pixels(20.0))
                                    .class("solo_btn");

                                    Element::new(cx).class("button_group_separator");

                                    Button::new(
                                        cx,
                                        |_| {},
                                        |cx| Label::new(cx, "M").bottom(Pixels(3.0)),
                                    )
                                    .child_space(Stretch(1.0))
                                    .width(Pixels(20.0))
                                    .height(Pixels(20.0))
                                    .class("mute_btn");
                                })
                                .class("button_group")
                                .left(Pixels(5.0))
                                .height(Pixels(20.0))
                                .width(Auto);
                            })
                            .height(Auto)
                            .top(Pixels(5.0))
                            .bottom(Stretch(1.0));
                        })
                        .width(Auto)
                        .left(Stretch(1.0))
                        .right(Pixels(3.0));

                        // TODO: Make decibel meter widget.
                        Element::new(cx).width(Pixels(12.0)).class("db_meter").space(Pixels(4.0));
                    } else {
                        // Compact view

                        VStack::new(cx, |cx| {
                            Label::new(cx, &state.name).class("name");
                        })
                        .left(Pixels(2.0))
                        .child_space(Pixels(4.0));

                        HStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Icon::new(cx, IconCode::Automation, 18.0, 16.0),
                                )
                                .class("icon_btn");
                            })
                            .class("button_group")
                            .height(Pixels(20.0))
                            .width(Auto);

                            HStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Icon::new(cx, IconCode::Record, 18.0, 16.0),
                                )
                                .class("icon_btn");
                            })
                            .class("button_group")
                            .left(Pixels(5.0))
                            .height(Pixels(20.0))
                            .width(Auto);

                            HStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Label::new(cx, "S").bottom(Pixels(3.0)),
                                )
                                .child_space(Stretch(1.0))
                                .width(Pixels(20.0))
                                .height(Pixels(20.0))
                                .class("solo_btn");

                                Element::new(cx).class("button_group_separator");

                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Label::new(cx, "M").bottom(Pixels(3.0)),
                                )
                                .child_space(Stretch(1.0))
                                .width(Pixels(20.0))
                                .height(Pixels(20.0))
                                .class("mute_btn");
                            })
                            .class("button_group")
                            .left(Pixels(5.0))
                            .height(Pixels(20.0))
                            .width(Auto);
                        })
                        .left(Stretch(1.0))
                        .right(Pixels(3.0))
                        .height(Auto)
                        .top(Pixels(5.0))
                        .bottom(Stretch(1.0));

                        // TODO: Make decibel meter widget.
                        Element::new(cx).width(Pixels(12.0)).class("db_meter").space(Pixels(4.0));
                    }
                })
                .class("background")
                .toggle_class("selected", state.selected)
                .height(Pixels(state.height));

                if state.selected {
                    header_view.border_color(state.color.into_color());
                }
            });
        })
        .height(Auto)
    }
}

enum InternalTrackHeaderEvent {
    StartResizeDrag,
    StopResizeDrag,
}

impl<L> View for TrackHeaderView<L>
where
    L: Lens<Target = BoundTrackHeaderState>,
{
    fn element(&self) -> Option<&'static str> {
        Some("trackheader")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|resize_drag_event, event| match resize_drag_event {
            InternalTrackHeaderEvent::StartResizeDrag => {
                self.is_resize_dragging = true;
                cx.capture();
                cx.lock_cursor_icon();
                event.consume();
            }

            InternalTrackHeaderEvent::StopResizeDrag => {
                self.is_resize_dragging = false;
                cx.release();
                cx.unlock_cursor_icon();
                event.consume()
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseMove(_, y) => {
                if self.is_resize_dragging {
                    let current = cx.current();
                    let posy = cx.cache.get_posy(current);
                    let old_height = cx.cache.get_height(current);
                    let dpi = cx.scale_factor();

                    let new_height = if self.is_master_track {
                        // Resize master track from the top.
                        old_height + ((posy - *y) / dpi)
                    } else {
                        // Resize all other tracks from the bottom.
                        (*y - posy) / dpi
                    };

                    (self.on_event)(cx, TrackHeaderEvent::DragResized(new_height));
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(InternalTrackHeaderEvent::StopResizeDrag);
            }

            WindowEvent::Press { .. } => {
                (self.on_event)(cx, TrackHeaderEvent::Selected);
                cx.release();
            }

            _ => {}
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrackHeaderEvent {
    DragResized(f32),
    Selected,
}