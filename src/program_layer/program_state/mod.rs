mod channel;
mod clip;
mod hrack_effect;
mod panel;
mod pattern;
mod timeline_grid;

pub use channel::*;
pub use clip::*;
pub use hrack_effect::*;
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

    pub patterns: Vec<PatternState>,

    /// The state of the timeline grid.
    ///
    /// (This does not contain the state of the clips.)
    pub timeline_grid: TimelineGridState,

    /// State of the UI panels.
    ///
    /// This is visual state that is used by the UI and must be serialized.
    pub panels: PanelState,
}

impl Model for ProgramState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|channel_event, meta| match channel_event {
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

        self.panels.event(cx, event);
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

#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub enum NotificationLogType {
    Error(String),
    Info(String),
}
