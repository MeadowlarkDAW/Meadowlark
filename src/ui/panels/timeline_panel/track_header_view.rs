use vizia::prelude::*;

use crate::state_system::working_state::track_headers_panel_state::{
    TrackHeaderState, DEFAULT_TRACK_HEADER_HEIGHT, MIN_TRACK_HEADER_HEIGHT,
};
use crate::ui::generic_views::knob::{KnobView, KnobViewStyle};
use crate::ui::generic_views::virtual_slider::{
    VirtualSliderDirection, VirtualSliderEvent, VirtualSliderMode, VirtualSliderScalars,
};
use crate::ui::generic_views::{icon, IconCode};

pub struct TrackHeaderView<L: Lens> {
    lens: L,
    is_resize_dragging: bool,
    on_event: Box<dyn Fn(&mut EventContext, TrackHeaderEvent)>,
    is_master_track: bool,
}

impl<L> TrackHeaderView<L>
where
    L: Lens<Target = TrackHeaderState>,
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
        .build(cx, |cx| {
            HStack::new(cx, |cx| {
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
                            .font_size(17.0)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .width(Pixels(16.0))
                            .height(Pixels(16.0))
                            .position_type(PositionType::SelfDirected)
                            .class("grip");
                    }
                })
                .width(Pixels(16.0))
                .background_color(lens.clone().map(|s| s.color.into_color()));

                /*
                let lens_clone = lens.clone();
                VStack::new(cx, move |cx| {
                    Label::new(cx, lens_clone.clone().map(|s| s.name.clone())).class("name");

                    Binding::new(cx, lens_clone.clone().map(|s| s.type_), move |cx, state| {
                        let type_ = state.get(cx);

                        // TODO: Fix icon sizes,
                        let (icon_symbol, icon_size) = match type_ {
                            TrackHeaderType::Master => (IconCode::MasterTrack, 20.0),
                            TrackHeaderType::Audio => (IconCode::Soundwave, 20.0),
                            TrackHeaderType::Synth => (IconCode::Piano, 16.0),
                        };

                        icon(cx, icon_symbol, 21.0, icon_size)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .display(lens_clone.clone().map(|s| s.height >= THRESHOLD_HEIGHT));
                    });
                })
                .left(Pixels(2.0))
                .child_space(Pixels(4.0));
                */

                Label::new(cx, lens.clone().map(|s| s.name.clone()))
                    .class("name")
                    .left(Pixels(5.0))
                    .top(Pixels(2.0));

                HStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Automation, 18.0, 16.0))
                            .class("icon_btn");
                    })
                    .class("button_group")
                    .height(Pixels(20.0))
                    .width(Auto);

                    HStack::new(cx, |cx| {
                        Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Record, 18.0, 16.0))
                            .class("icon_btn");
                    })
                    .class("button_group")
                    .left(Pixels(5.0))
                    .height(Pixels(20.0))
                    .width(Auto);

                    HStack::new(cx, |cx| {
                        Button::new(cx, |_| {}, |cx| Label::new(cx, "S").bottom(Pixels(3.0)))
                            .child_space(Stretch(1.0))
                            .width(Pixels(20.0))
                            .height(Pixels(20.0))
                            .class("solo_btn");

                        Element::new(cx).class("button_group_separator");

                        Button::new(cx, |_| {}, |cx| Label::new(cx, "M").bottom(Pixels(3.0)))
                            .child_space(Stretch(1.0))
                            .width(Pixels(20.0))
                            .height(Pixels(20.0))
                            .class("mute_btn");
                    })
                    .class("button_group")
                    .left(Pixels(5.0))
                    .height(Pixels(20.0))
                    .width(Auto);

                    KnobView::new(
                        cx,
                        lens.clone().map(|s| s.volume),
                        VirtualSliderMode::Continuous,
                        VirtualSliderDirection::Vertical,
                        VirtualSliderScalars::default(),
                        Pixels(7.0),
                        false,
                        KnobViewStyle::default(),
                        |cx, event| match event {
                            VirtualSliderEvent::Changed(value_normalized) => {
                                cx.emit(InternalTrackHeaderEvent::SetVolumeNormalized(
                                    value_normalized,
                                ));
                            }
                            _ => {}
                        },
                    )
                    .left(Pixels(5.0))
                    .right(Pixels(2.0))
                    .top(Pixels(1.0))
                    .width(Pixels(20.0))
                    .height(Pixels(20.0));
                })
                .left(Stretch(1.0))
                .height(Auto)
                .top(Pixels(2.0));

                // TODO: Make decibel meter widget.
                Element::new(cx).width(Pixels(12.0)).class("db_meter").space(Pixels(4.0));
            })
            .class("background")
            .toggle_class("selected", lens.clone().map(|s| s.selected))
            .height(lens.map(|s| Pixels(s.height)));
        })
        .height(Auto)
    }
}

enum InternalTrackHeaderEvent {
    StartResizeDrag,
    StopResizeDrag,
    SetVolumeNormalized(f32),
    SetPanNormalized(f32),
}

impl<L> View for TrackHeaderView<L>
where
    L: Lens<Target = TrackHeaderState>,
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

            InternalTrackHeaderEvent::SetVolumeNormalized(volume_normalized) => {
                (self.on_event)(cx, TrackHeaderEvent::SetVolumeNormalized(*volume_normalized));
            }
            InternalTrackHeaderEvent::SetPanNormalized(pan_normalized) => {
                (self.on_event)(cx, TrackHeaderEvent::SetPanNormalized(*pan_normalized));
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseMove(_, y) => {
                if self.is_resize_dragging {
                    let current = cx.current();
                    let posy = cx.cache.get_posy(current);
                    let old_height = cx.cache.get_height(current);
                    let scale_factor = cx.style.dpi_factor as f32;

                    let new_height = if self.is_master_track {
                        // Resize master track from the top.
                        (old_height + (posy - *y)) / scale_factor
                    } else {
                        // Resize all other tracks from the bottom.
                        (*y - posy) / scale_factor
                    };

                    (self.on_event)(cx, TrackHeaderEvent::Resized(new_height));
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                cx.emit(InternalTrackHeaderEvent::StopResizeDrag);
            }

            WindowEvent::Press { .. } => {
                if !self.is_resize_dragging {
                    cx.release();
                    (self.on_event)(cx, TrackHeaderEvent::Selected);
                }
            }

            WindowEvent::MouseDoubleClick(button) if *button == MouseButton::Left => {
                cx.release();

                let current_height = self.lens.get(cx).height;

                if current_height != DEFAULT_TRACK_HEADER_HEIGHT {
                    // If double-clicked and the height of the track is not
                    // the default height, then reset to default height.
                    (self.on_event)(cx, TrackHeaderEvent::Resized(DEFAULT_TRACK_HEADER_HEIGHT));
                } else {
                    // Else if the height of the track is already the default
                    // height, then minimize the height.
                    (self.on_event)(cx, TrackHeaderEvent::Resized(MIN_TRACK_HEADER_HEIGHT));
                }
            }

            _ => {}
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrackHeaderEvent {
    Resized(f32),
    Selected,
    SetVolumeNormalized(f32),
    SetPanNormalized(f32),
}
