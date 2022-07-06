pub mod top_bar;
pub use top_bar::*;

pub mod bottom_bar;
pub use bottom_bar::*;

pub mod left_bar;
pub use left_bar::*;

pub mod browser;
pub use browser::*;

pub mod channels;
pub use channels::*;

pub mod clip;
pub use clip::*;

pub mod timeline;
pub use timeline::*;

pub mod piano_roll;
pub use piano_roll::*;

pub mod hrack_effect;
pub use hrack_effect::*;

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub channel_rack_orientation: ChannelRackOrientation,
    pub hide_clips: bool,
    pub hide_piano_roll: bool,
    pub browser_width: f32,
    pub show_browser: bool,
}

pub enum PanelEvent {
    ToggleChannelRackOrientation,
    ToggleClips,
    ShowClips,
    TogglePianoRoll,
    SetBrowserWidth(f32),
    ToggleBrowser,
}

impl Model for PanelState {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        event.map(|channel_rack_event, _| match channel_rack_event {
            PanelEvent::ToggleChannelRackOrientation => {
                if self.channel_rack_orientation == ChannelRackOrientation::Horizontal {
                    self.channel_rack_orientation = ChannelRackOrientation::Vertical;
                } else {
                    self.channel_rack_orientation = ChannelRackOrientation::Horizontal;
                }
            }

            PanelEvent::ToggleClips => {
                self.hide_clips ^= true;
            }

            PanelEvent::ShowClips => {
                self.hide_clips = false;
            }

            PanelEvent::TogglePianoRoll => {
                self.hide_piano_roll ^= true;
            }

            PanelEvent::SetBrowserWidth(width) => {
                self.browser_width = *width;
                if self.browser_width < 50.0 {
                    self.show_browser = false;
                } else {
                    self.show_browser = true;
                }
            }

            PanelEvent::ToggleBrowser => {
                self.show_browser ^= true;
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data, Serialize, Deserialize)]
pub enum ChannelRackOrientation {
    Horizontal,
    Vertical,
}

impl Default for ChannelRackOrientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

impl From<ChannelRackOrientation> for bool {
    fn from(orientation: ChannelRackOrientation) -> bool {
        match orientation {
            ChannelRackOrientation::Vertical => true,
            ChannelRackOrientation::Horizontal => false,
        }
    }
}
