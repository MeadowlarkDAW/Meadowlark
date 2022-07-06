use vizia::prelude::*;

use crate::ui::{AppData, Panel, UiState};

use super::clip::{AudioClipState, AutomationClipState, ClipState, PianoRollClipState};
use super::hrack_effect::HRackEffectState;
use super::{PanelEvent, PanelState};

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub enum ChannelBaseColor {
    /// This is an index into a bunch of preset colors that are defined
    /// by the current theme.
    Preset(u16),
    Color(Color),
}

impl From<ChannelBaseColor> for Color {
    fn from(col: ChannelBaseColor) -> Self {
        match col {
            ChannelBaseColor::Preset(_) => Color::red(),
            ChannelBaseColor::Color(col) => col,
        }
    }
}

impl From<Color> for ChannelBaseColor {
    fn from(col: Color) -> Self {
        ChannelBaseColor::Color(col)
    }
}

// Helper function for recursively collecting the indices of selected channels
fn select_channel(channel_data: &Vec<ChannelState>, index: usize, selected: &mut Vec<usize>) {
    if let Some(data) = channel_data.get(index) {
        selected.push(index);
        for subchannel in data.subchannels.iter() {
            select_channel(channel_data, *subchannel, selected);
        }
    }
}

/// A "channel" refers to a mixer channel.
#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct ChannelStates {
    pub channels: Vec<ChannelState>,
}

impl Model for ChannelStates {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|channel_event, _| match channel_event {
            ChannelEvent::SelectChannel(index) => {
                for channel_data in self.channels.iter_mut() {
                    channel_data.selected = false;
                }

                let mut selected = vec![];

                select_channel(&self.channels, *index, &mut selected);

                for idx in selected.iter() {
                    if let Some(channel_data) = self.channels.get_mut(*idx) {
                        channel_data.selected = true;
                    }
                }
            }
        });
    }
}

/// A "channel" refers to a mixer channel.
#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct ChannelState {
    /// The channel name
    pub name: String,

    /// The channel color
    pub color: ChannelBaseColor,

    /// Subchannels of this Channel
    pub subchannels: Vec<usize>,

    /// Flag indicating whether the channel is currently selected in UI
    #[serde(skip)]
    pub selected: bool,

    /// The audio clips assigned to this channel.
    pub audio_clips: Vec<AudioClipState>,
    /// The audio clips assigned to this channel.
    pub piano_roll_clips: Vec<PianoRollClipState>,
    /// The audio clips assigned to this channel.
    pub automation_clips: Vec<AutomationClipState>,

    // TODO: Use some kind of tree structure instead of a Vec once we
    // implement container effects.
    pub effects: Vec<HRackEffectState>,

    /// The index to the channel that this channel is routed to.
    ///
    /// The master channel is always at index 0.
    pub routed_to: usize,

    /// The normalized value of the channel's output gain in the range [0.0, 1.0].
    pub out_gain_normalized: f64,

    /// The normalized value of the channel's output pan in the range [0.0, 1.0].
    pub out_pan_normalized: f64,

    /// The currently displayed value for the channel's output gain (i.e. "-12.0dB").
    pub out_gain_display: String,

    /// The currently displayed value for the channel's output pan (i.e. "75R").
    pub out_pan_display: String,

    /// True if this channel is currently being "soloed".
    pub soloed: bool,

    /// True if this channel is currently being muted.
    pub muted: bool,
    // TODO: Sends
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            name: String::from("Channel"),
            color: ChannelBaseColor::Color(Color::red()),
            subchannels: vec![],
            selected: false,
            audio_clips: vec![],
            piano_roll_clips: vec![],
            automation_clips: vec![],
            effects: vec![],
            routed_to: 0,
            out_gain_normalized: 1.0,
            out_pan_normalized: 0.5,
            out_gain_display: String::from("0dB"),
            out_pan_display: String::from("0"),
            soloed: false,
            muted: false,
        }
    }
}

pub enum ChannelEvent {
    SelectChannel(usize),
}

pub fn channels(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Container for channels and clips.
        // Although this is a vstack we're using css to switch between horizontal and vertical layouts.
        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // TODO: Make this resizable when channel rack orientation is vertical.
                Panel::new(
                    cx,
                    |cx| {
                        Label::new(cx, "CHANNEL RACK").class("small");

                        // Button to toggle the orientation of the channels & clips.
                        // TODO: Replace with toggle button when we have a design for it.
                        // TODO: Replace label with icon once we have an icon for it.
                        Button::new(
                            cx,
                            |cx| {
                                cx.emit(PanelEvent::ToggleChannelRackOrientation);
                                cx.emit(PanelEvent::ShowClips);
                            },
                            |cx| Label::new(cx, "A"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .left(Stretch(1.0));

                        // Button to hide the clips panel.
                        // TODO: Replace with toggle button when we have a design for it.
                        // TODO: Replace label with icon once we have an icon for it.
                        Button::new(
                            cx,
                            |cx| cx.emit(PanelEvent::ToggleClips),
                            |cx| Label::new(cx, "B"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .right(Pixels(10.0));
                    },
                    |cx| {
                        ScrollView::new(cx, 0.0, 0.0, false, false, |cx| {
                            // Master Channel
                            HStack::new(cx, |cx| {
                                Element::new(cx)
                                    .width(Pixels(14.0))
                                    .background_color(Color::from("#D4D5D5"))
                                    .class("bar");

                                VStack::new(cx, |cx| {
                                    Label::new(
                                        cx,
                                        AppData::state.then(
                                            UiState::channels.then(
                                                ChannelStates::channels
                                                    .index(0)
                                                    .then(ChannelState::name),
                                            ),
                                        ),
                                    );
                                });
                            })
                            .class("channel")
                            .toggle_class(
                                "selected",
                                AppData::state.then(UiState::channels.then(
                                    ChannelStates::channels.index(0).then(ChannelState::selected),
                                )),
                            )
                            .on_press(move |cx| cx.emit(ChannelEvent::SelectChannel(0)));

                            // Other Channels
                            List::new(
                                cx,
                                AppData::state.then(
                                    UiState::channels.then(
                                        ChannelStates::channels
                                            .index(0)
                                            .then(ChannelState::subchannels),
                                    ),
                                ),
                                |cx, _, item| {
                                    channel(
                                        cx,
                                        AppData::state
                                            .then(UiState::channels.then(ChannelStates::channels)),
                                        item.get(cx),
                                        0,
                                    );
                                },
                            )
                            .row_between(Pixels(4.0));
                        })
                        .class("channels_content");
                    },
                )
                .width(Pixels(225.0))
                .class("instruments");

                clips(cx);
            })
            .overflow(Overflow::Hidden);

            clips(cx);
        })
        .toggle_class(
            "vertical",
            AppData::state
                .then(UiState::panels.then(PanelState::channel_rack_orientation))
                .map(|&val| val.into()),
        )
        .toggle_class("hidden", AppData::state.then(UiState::panels.then(PanelState::hide_clips)))
        .class("channels");
    })
    .class("channel_rack");
}

fn clips(cx: &mut Context) {
    // Clips panel (Horizontal)
    Panel::new(
        cx,
        |cx| {
            Label::new(cx, "PATTERNS").class("small").text_wrap(false);
        },
        |cx| {
            ScrollView::new(cx, 0.0, 0.0, false, false, |cx| {
                // List of clips. Visibility is determined by whether the associated channel is selected.
                List::new(cx, AppData::state.then(UiState::clips), |cx, _, clip| {
                    let channel_index = clip.get(cx).channel;

                    VStack::new(cx, |cx| {
                        Label::new(cx, clip.then(ClipState::name))
                            .text_wrap(false)
                            .background_color(
                                AppData::state
                                    .then(
                                        UiState::channels.then(
                                            ChannelStates::channels
                                                .index(channel_index)
                                                .then(ChannelState::color),
                                        ),
                                    )
                                    .map(|col| col.clone().into()),
                            );
                    })
                    .visibility(
                        AppData::state.then(
                            UiState::channels
                                .then(ChannelStates::channels.index(channel_index))
                                .then(ChannelState::selected),
                        ),
                    )
                    .class("clip");
                })
                .child_space(Pixels(4.0));
            });
        },
    )
    .class("clips")
    .checked(AppData::state.then(UiState::panels.then(PanelState::hide_clips)));
}

pub fn channel<L: Lens<Target = Vec<ChannelState>>>(
    cx: &mut Context,
    root: L,
    index: usize,
    level: usize,
) where
    <L as Lens>::Source: Model,
{
    VStack::new(cx, |cx| {
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
            .on_press(move |cx| cx.emit(ChannelEvent::SelectChannel(index)));

            HStack::new(cx, |cx| {
                //Spacer
                Element::new(cx).class("group_bar");
                VStack::new(cx, |cx| {
                    for idx in data.subchannels.iter() {
                        let new_root = new_root.clone();
                        channel(cx, new_root, *idx, level + 1);
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
