use vizia::prelude::*;

// TODO - Move this to its own file with other local UI state
#[derive(Debug, Lens, Clone)]
pub struct PanelState {
    pub channel_rack_orientation: ChannelRackOrientation,
    pub hide_clips: bool,
    pub hide_piano_roll: bool,
    pub browser_width: f32,
    pub hide_browser: bool,
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
                    self.hide_browser = true;
                    self.browser_width = 0.0;
                } else {
                    self.hide_browser = false;
                }
            }

            PanelEvent::ToggleBrowser => {
                self.hide_browser ^= true;
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
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
