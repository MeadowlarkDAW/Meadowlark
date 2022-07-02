use vizia::{prelude::*, state::RatioLens};

mod keymap;
use keymap::*;

use crate::{
    program_layer::{
        program_state::{ChannelEvent, ChannelState, PanelEvent, PanelState, PatternState},
        ProgramLayer, ProgramState,
    },
    ui_layer::Panel,
};

pub fn channels(cx: &mut Context) {
    channels_keymap(cx);

    VStack::new(cx, |cx| {
        // Container for channels and patterns.
        // Although this is a vstack we're using css to switch between horizontal and vertical layouts.
        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // TODO: Make this resizable when channel rack orientation is vertical.
                Panel::new(
                    cx,
                    |cx| {
                        Label::new(cx, "CHANNEL RACK").class("small");

                        // Button to toggle the orientation of the channels & patterns.
                        // TODO: Replace with toggle button when we have a design for it.
                        // TODO: Replace label with icon once we have an icon for it.
                        Button::new(
                            cx,
                            |cx| {
                                cx.emit(PanelEvent::ToggleChannelRackOrientation);
                                cx.emit(PanelEvent::ShowPatterns);
                            },
                            |cx| Label::new(cx, "A"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .left(Stretch(1.0));

                        // Button to hide the patterns panel.
                        // TODO: Replace with toggle button when we have a design for it.
                        // TODO: Replace label with icon once we have an icon for it.
                        Button::new(
                            cx,
                            |cx| cx.emit(PanelEvent::TogglePatterns),
                            |cx| Label::new(cx, "B"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .right(Pixels(10.0));
                    },
                    |cx| {
                        ScrollData {
                            scroll_x: 0.0,
                            scroll_y: 0.0,
                            child_x: 0.0,
                            child_y: 0.0,
                            parent_x: 0.0,
                            parent_y: 0.0,
                        }
                        .build(cx);

                        HStack::new(cx, |cx| {
                            ScrollView::custom(cx, false, false, ScrollData::root, |cx| {
                                // Master Channel
                                HStack::new(cx, |cx| {
                                    // Left color bar
                                    Element::new(cx)
                                        .width(Pixels(14.0))
                                        .background_color(Color::from("#D4D5D5"))
                                        .class("bar");
                                    // Master channel controls
                                    VStack::new(cx, |cx| {
                                        // Title
                                        Label::new(
                                            cx,
                                            ProgramLayer::state.then(
                                                ProgramState::channels
                                                    .index(0)
                                                    .then(ChannelState::name),
                                            ),
                                        );
                                    });
                                })
                                .class("channel")
                                .toggle_class(
                                    "selected",
                                    ProgramLayer::state.then(
                                        ProgramState::channels
                                            .index(0)
                                            .then(ChannelState::selected),
                                    ),
                                )
                                .on_press(move |cx| cx.emit(ChannelEvent::SelectChannel(0)));

                                // Other Channels
                                List::new(
                                    cx,
                                    ProgramLayer::state.then(
                                        ProgramState::channels
                                            .index(0)
                                            .then(ChannelState::subchannels),
                                    ),
                                    |cx, _, item| {
                                        let index = item.get(cx);

                                        Channel::new(
                                            cx,
                                            ProgramLayer::state.then(ProgramState::channels),
                                            item.get(cx),
                                            0,
                                        );
                                        // Movement indicator
                                        // Element::new(cx).height(Pixels(4.0)).width(Stretch(1.0))
                                        // .class("move-indicator")
                                        // .bind(ProgramLayer::state.then(ProgramState::dragging_channel), move |handle, dragging|{
                                        //     handle.bind(ProgramLayer::state.then(ProgramState::channels), move |handle, channels|{
                                        //         if let Some(dragging) = dragging.get(handle.cx) {
                                        //             let drag_channel_state = channels.index(dragging).get(handle.cx);
                                        //             let channel_state = channels.index(index).get(handle.cx);
                                        //             println!("{:?} {:?}", drag_channel_state.path, channel_state.path);
                                        //             handle.toggle_class("drag", !channel_state.path.starts_with(drag_channel_state.path) && dragging != index);
                                        //         }
                                        //     });

                                        // })
                                        // .on_release(move |cx|{
                                        //     cx.emit(ChannelEvent::DropChannel(index));
                                        // });
                                    },
                                )
                                .row_between(Pixels(4.0));
                            })
                            .class("channels_content");

                            // A custom scrollbar used to scroll the custom views vertically.
                            Scrollbar::new(
                                cx,
                                ScrollData::scroll_y,
                                RatioLens::new(ScrollData::parent_y, ScrollData::child_y),
                                Orientation::Vertical,
                                |cx, scroll| {
                                    cx.emit(ScrollEvent::SetY(scroll));
                                },
                            )
                            .width(Units::Pixels(14.0))
                            .height(Stretch(1.0));
                        });
                    },
                )
                .width(Pixels(225.0))
                .class("instruments");

                patterns(cx);
            })
            .overflow(Overflow::Hidden);

            patterns(cx);
        })
        .toggle_class(
            "vertical",
            ProgramLayer::state
                .then(ProgramState::panels.then(PanelState::channel_rack_orientation))
                .map(|&val| val.into()),
        )
        .toggle_class(
            "hidden",
            ProgramLayer::state.then(ProgramState::panels.then(PanelState::hide_patterns)),
        )
        .class("channels");
    })
    .class("channel_rack");
}

fn patterns(cx: &mut Context) {
    // Patterns panel (Horizontal)
    Panel::new(
        cx,
        |cx| {
            Label::new(cx, "PATTERNS").class("small").text_wrap(false);
        },
        |cx| {
            ScrollView::new(cx, 0.0, 0.0, false, false, |cx| {
                // List of patterns. Visibility is determined by whether the associated channel is selected.
                List::new(
                    cx,
                    ProgramLayer::state.then(ProgramState::patterns),
                    |cx, _, pattern| {
                        let channel_index = pattern.get(cx).channel;

                        VStack::new(cx, |cx| {
                            Label::new(cx, pattern.then(PatternState::name))
                                .text_wrap(false)
                                .background_color(
                                    ProgramLayer::state.then(
                                        ProgramState::channels
                                            .index(channel_index)
                                            .then(ChannelState::color)
                                            .map(|col| col.clone().into()),
                                    ),
                                );
                        })
                        .visibility(
                            ProgramLayer::state.then(
                                ProgramState::channels
                                    .index(channel_index)
                                    .then(ChannelState::selected),
                            ),
                        )
                        .class("pattern");
                    },
                )
                .child_space(Pixels(4.0));
            });
        },
    )
    .class("patterns")
    .checked(ProgramLayer::state.then(ProgramState::panels.then(PanelState::hide_patterns)));
}

pub struct Channel {
    channel_index: usize,
}

impl Channel {
    pub fn new<L: Lens<Target = Vec<ChannelState>>>(
        cx: &mut Context,
        root: L,
        index: usize,
        level: usize,
    ) where
        <L as Lens>::Source: Model,
    {
        Self { channel_index: index }
            .build(cx, |cx| {
                let new_root = root.clone();
                Binding::new(cx, root.index(index), move |cx, chnl| {
                    let data = chnl.get(cx);

                    let col: Color = data.color.into();

                    HStack::new(cx, |cx| {
                        let is_grouped = !data.subchannels.is_empty();
                        Element::new(cx)
                            .width(Pixels(14.0))
                            .background_color(col)
                            .class("bar")
                            .toggle_class("grouped", is_grouped);

                        VStack::new(cx, |cx| {
                            Label::new(cx, chnl.then(ChannelState::name));
                        });
                    })
                    .class("channel")
                    .toggle_class("selected", data.selected)
                    .on_press(move |cx| {
                        cx.emit(ChannelEvent::SelectChannel(index));
                        // println!("Start Drag: {}", index);
                        // cx.emit(ChannelEvent::DragChannel(index));
                    });

                    HStack::new(cx, |cx| {
                        //Spacer
                        Element::new(cx).class("group_bar");
                        VStack::new(cx, |cx| {
                            for idx in data.subchannels.iter() {
                                let new_root = new_root.clone();

                                Channel::new(cx, new_root, *idx, level + 1);
                                // let ix = *idx;
                                // Element::new(cx).height(Pixels(4.0)).width(Stretch(1.0))
                                //     .class("move-indicator")
                                //     .bind(ProgramLayer::state.then(ProgramState::dragging_channel), move |handle, dragging|{
                                //         handle.bind(ProgramLayer::state.then(ProgramState::channels), move |handle, channels|{
                                //             if let Some(dragging) = dragging.get(handle.cx) {
                                //                 let drag_channel_state = channels.index(dragging).get(handle.cx);
                                //                 let channel_state = channels.index(index).get(handle.cx);
                                //                 println!("{:?} {:?}", drag_channel_state.path, channel_state.path);
                                //                 handle.toggle_class("drag", !channel_state.path.starts_with(drag_channel_state.path) && dragging != index);
                                //             }
                                //         });

                                //     })
                                //     .on_release(move |cx|{
                                //         cx.emit(ChannelEvent::DropChannel(ix));
                                //     });
                            }
                        })
                        .class("channel_group");
                    })
                    .border_radius_bottom_left(Pixels(2.0))
                    .background_color(col);
                });
            })
            .height(Auto);
    }
}

impl View for Channel {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDoubleClick(button) if *button == MouseButton::Left => {
                println!("Received double click event");
                cx.emit(ChannelEvent::SelectChannelGroup(self.channel_index));
            }

            _ => {}
        });
    }
}
