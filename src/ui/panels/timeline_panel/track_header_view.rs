use vizia::prelude::*;

use crate::state_system::app_state::{PaletteColor, TrackType, TracksState};
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
}

pub fn bound_state_from_tracks_state(
    tracks_state: &TracksState,
) -> (BoundTrackHeaderState, Vec<BoundTrackHeaderState>) {
    let master_track_header = BoundTrackHeaderState {
        name: "Master".into(),
        color: tracks_state.master_track_color,
        height: tracks_state.master_track_lane_height,
        type_: BoundTrackHeaderType::Master,
    };

    let track_headers: Vec<BoundTrackHeaderState> = tracks_state
        .tracks
        .iter()
        .map(|track_state| BoundTrackHeaderState {
            name: track_state.name.clone(),
            color: track_state.color,
            height: track_state.lane_height,
            type_: match track_state.type_ {
                TrackType::Audio => BoundTrackHeaderType::Audio,
                TrackType::Synth => BoundTrackHeaderType::Synth,
            },
        })
        .collect();

    (master_track_header, track_headers)
}

// TODO: Double-click to reset to default height.

pub struct TrackHeaderView<L: Lens> {
    lens: L,
    is_resize_dragging: bool,
    on_resize_drag: Box<dyn Fn(&mut EventContext, f32)>,
}

impl<L> TrackHeaderView<L>
where
    L: Lens<Target = BoundTrackHeaderState>,
{
    pub fn new<'a>(
        cx: &'a mut Context,
        lens: L,
        on_resize_drag: impl Fn(&mut EventContext, f32) + 'static,
    ) -> Handle<'a, Self> {
        Self {
            lens: lens.clone(),
            is_resize_dragging: false,
            on_resize_drag: Box::new(on_resize_drag),
        }
        .build(cx, move |cx| {
            Binding::new(cx, lens, move |cx, state| {
                let state = state.get(cx);

                HStack::new(cx, |cx| {
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
                                cx.emit(TrackHeaderEvent::StartResizeDrag);
                            }
                        });

                    HStack::new(cx, |cx| {
                        Label::new(cx, IconCode::GripVertical)
                            .font_size(19.0)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .width(Pixels(20.0))
                            .height(Pixels(20.0))
                            .position_type(PositionType::SelfDirected)
                            .class("grip")
                            .font("meadowlark-icons");
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
                .height(Pixels(state.height));
            });
        })
        .height(Auto)
    }
}

pub enum TrackHeaderEvent {
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
            TrackHeaderEvent::StartResizeDrag => {
                self.is_resize_dragging = true;
                cx.capture();
                cx.lock_cursor_icon();
                event.consume();
            }

            TrackHeaderEvent::StopResizeDrag => {
                self.is_resize_dragging = false;
                cx.release();
                cx.unlock_cursor_icon();
                event.consume()
            }
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::MouseMove(_, y) => {
                if self.is_resize_dragging {
                    let current = cx.current();
                    let posy = cx.cache.get_posy(current);
                    let dpi = cx.scale_factor();
                    let new_height = (*y - posy) / dpi;
                    (self.on_resize_drag)(cx, new_height);
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(TrackHeaderEvent::StopResizeDrag);
            }

            _ => {}
        });
    }
}
