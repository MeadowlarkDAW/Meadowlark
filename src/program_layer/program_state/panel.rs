use vizia::prelude::*;

// TODO - Move this to its own file with other local UI state
#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub channel_rack_orientation: ChannelRackOrientation,
    pub hide_patterns: bool,
    pub hide_piano_roll: bool,
    pub browser_width: f32,
    pub show_browser: bool,
}

pub enum PanelEvent {
    ToggleChannelRackOrientation,
    TogglePatterns,
    ShowPatterns,
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

            PanelEvent::TogglePatterns => {
                self.hide_patterns ^= true;
            }

            PanelEvent::ShowPatterns => {
                self.hide_patterns = false;
            }

            PanelEvent::TogglePianoRoll => {
                self.hide_piano_roll ^= true;
            }

            PanelEvent::SetBrowserWidth(width) => {
                self.browser_width = *width;
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
