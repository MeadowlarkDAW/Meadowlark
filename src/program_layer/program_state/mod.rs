mod browser;
mod channel;
mod clip;
mod hrack_effect;
mod lane_states;
mod panel;
mod pattern;
mod timeline_grid;

use std::{ops::Index, path::PathBuf, vec};

pub use browser::*;
pub use channel::*;
pub use clip::*;
pub use hrack_effect::*;
pub use lane_states::*;
pub use panel::*;
pub use pattern::*;
pub use timeline_grid::*;

use vizia::prelude::*;

/// The state of the whole program.
///
/// Unless explicitely stated, the UI may NOT directly mutate the state of any
/// of these variables. It is intended for the UI to call the methods on this
/// struct in order to mutate state.
#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub struct ProgramState {
    /// True if a backend engine is currently running, false if not.
    ///
    /// Nothing except the settings menu can be accessed when this is false.
    pub engine_running: bool,

    /// This contains all of the text for any notifications (errors or otherwise)
    /// that are being displayed to the user.
    ///
    /// The UI may mutate this directly without an event.
    pub notification_log: Vec<NotificationLogType>,

    /// A "channel" refers to a mixer channel.
    ///
    /// This also contains the state of all clips.
    pub channels: Vec<ChannelState>,

    // Index of channel being dragged
    #[serde(skip)]
    pub dragging_channel: Option<usize>,

    pub patterns: Vec<PatternState>,

    /// The state of the timeline grid.
    ///
    /// (This does not contain the state of the clips.)
    pub timeline_grid: TimelineGridState,

    pub browser: BrowserState,

    /// State of the UI panels.
    ///
    /// This is visual state that is used by the UI and must be serialized.
    pub panels: PanelState,
}

impl Model for ProgramState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|channel_event, _| match channel_event {
            // Select a single channel
            ChannelEvent::SelectChannel(index) => {
                deselect_channels(&mut self.channels);

                if let Some(channel_data) = self.channels.get_mut(*index) {
                    channel_data.selected = true;
                }
            }

            // Select a channel and any children in the same group
            ChannelEvent::SelectChannelGroup(index) => {
                println!("Select channel group: {}", index);
                deselect_channels(&mut self.channels);

                let mut selected = vec![];

                select_channel(&self.channels, *index, &mut selected);

                for idx in selected.iter() {
                    if let Some(channel_data) = self.channels.get_mut(*idx) {
                        channel_data.selected = true;
                    }
                }
            }

            // Add a new channel to the channels panel
            ChannelEvent::AddChannel => {
                deselect_channels(&mut self.channels);

                let channel_id = self.channels.len();

                // Create a new channel
                self.channels.push(ChannelState {
                    name: String::from("New Channel"),
                    path: PathBuf::from("New Channel"),
                    color: ChannelBaseColor::Color(Color::rgb(200, 50, 50)),
                    selected: true,
                    ..Default::default()
                });

                // Add new channel to master group
                if let Some(master) = self.channels.get_mut(0) {
                    master.subchannels.push(channel_id);
                }
            }

            // Remove the specified channel from the channels panel
            ChannelEvent::RemoveChannel => {} // TODO
                                              // ChannelEvent::DragChannel(index) => {
                                              //     self.dragging_channel = Some(*index);
                                              // }

                                              // TODO
                                              // ChannelEvent::DropChannel(index) => {
                                              //     println!("Drop: {:?} {}", self.dragging_channel, index);
                                              //     if let Some(dragging_channel) = self.dragging_channel {
                                              //         if *index == dragging_channel {
                                              //             return;
                                              //         }

                                              //         let drag_channel_path = self
                                              //             .channels
                                              //             .get(dragging_channel)
                                              //             .and_then(|state| Some(state.path.clone()));
                                              //         let drag_channel_parent = self
                                              //             .channels
                                              //             .get(dragging_channel)
                                              //             .and_then(|state| state.parent_channel)
                                              //             .unwrap();

                                              //         let mut mv = false;

                                              //         if let Some(drag_channel_path) = drag_channel_path {
                                              //             if let Some(drop_channel_state) = self.channels.get_mut(*index) {
                                              //                 println!("Do this");
                                              //                 if !drop_channel_state.path.starts_with(&drag_channel_path) {
                                              //                     let prev_parent = drag_channel_parent;
                                              //                     mv = true;
                                              //                     //drop_channel_state.subchannels.push(dragging_channel);
                                              //                 }
                                              //             }
                                              //         }

                                              //         if mv {
                                              //             // Remove from parent
                                              //             println!("Remove from parent: {}", drag_channel_parent);
                                              //             if let Some(parent_state) = self.channels.get_mut(drag_channel_parent) {
                                              //                 if let Some(pos) =
                                              //                     parent_state.subchannels.iter().position(|&x| x == dragging_channel)
                                              //                 {
                                              //                     parent_state.subchannels.remove(pos);
                                              //                 }
                                              //             }

                                              //             // Add to new parent
                                              //             if let Some(drop_channel_state) = self.channels.get_mut(*index) {
                                              //                 println!("Add to new parent");
                                              //                 drop_channel_state.subchannels.push(dragging_channel);
                                              //             }

                                              //             // Set new parent
                                              //             if let Some(drag_channel_state) = self.channels.get_mut(dragging_channel) {
                                              //                 println!("Set new parent");
                                              //                 drag_channel_state.parent_channel = Some(*index);
                                              //             }

                                              //             self.dragging_channel = None;
                                              //         }
                                              //     }
                                              // }
        });

        self.panels.event(cx, event);
        self.timeline_grid.event(cx, event);
        self.browser.event(cx, event);
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

// Helper function for deselecting all channels
fn deselect_channels(channel_data: &mut Vec<ChannelState>) {
    for channel in channel_data.iter_mut() {
        channel.selected = false;
    }
}

#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub enum NotificationLogType {
    Error(String),
    Info(String),
}
