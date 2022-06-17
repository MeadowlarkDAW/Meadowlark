use vizia::prelude::*;

use super::app_event::AppEvent;

// TODO - Move this somewhere probably
#[derive(Lens)]
pub struct AppData {
    pub channel_data: Vec<ChannelData>,
    pub pattern_data: Vec<PatternData>,
    pub track_data: Vec<TrackData>,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::SelectChannel(index) => {
                for channel_data in self.channel_data.iter_mut() {
                    channel_data.selected = false;
                }

                let mut selected = vec![];

                select_channel(&self.channel_data, *index, &mut selected);

                for idx in selected.iter() {
                    if let Some(channel_data) = self.channel_data.get_mut(*idx) {
                        channel_data.selected = true;
                    }
                }
            }
            AppEvent::SelectTrack(index) => {
                for (i, track_data) in self.track_data.iter_mut().enumerate() {
                    track_data.selected = i == *index;
                }
            }
            AppEvent::InsertTrack => {
                if let Some(index) = selected_track(&self.track_data) {
                    self.track_data[index].selected = false;
                }

                self.track_data.push(TrackData {
                    name: String::from("New Track"),
                    color: Color::from("#7D7D7D"),
                    selected: true,
                });
            }
            AppEvent::DuplicateSelectedTrack => {
                let (index, track) = {
                    if let Some(index) = selected_track(&self.track_data) {
                        let mut selected_track = &mut self.track_data[index];
                        let cloned_track = selected_track.clone();
                        selected_track.selected = false;
                        (index, cloned_track)
                    } else {
                        return;
                    }
                };

                self.track_data.insert(index, track);
            }
            AppEvent::MoveSelectedTrackUp => {
                if let Some(index) = selected_track(&self.track_data) {
                    if index > 0 {
                        self.track_data.swap(index, index - 1);
                    }
                }
            }
            AppEvent::MoveSelectedTrackDown => {
                if let Some(index) = selected_track(&self.track_data) {
                    if index < self.track_data.len() - 1 {
                        self.track_data.swap(index, index + 1);
                    }
                }
            }
            AppEvent::DeleteSelectedTrack => {
                if let Some(index) = selected_track(&self.track_data) {
                    self.track_data.remove(index);

                    if self.track_data.len() > 0 {
                        let new_index = index.min(self.track_data.len() - 1);
                        self.track_data[new_index].selected = true;
                    }
                }
            }
            AppEvent::SelectTrackAbove => {
                if let Some(index) = selected_track(&self.track_data) {
                    if index > 0 {
                        self.track_data[index].selected = false;
                        self.track_data[index - 1].selected = true;
                    }
                }
            }
            AppEvent::SelectTrackBelow => {
                if let Some(index) = selected_track(&self.track_data) {
                    if index < self.track_data.len() - 1 {
                        self.track_data[index].selected = false;
                        self.track_data[index + 1].selected = true;
                    }
                }
            }
            _ => {}
        });
    }
}

// Helper function to return the index of the selected track.
fn selected_track(track_data: &Vec<TrackData>) -> Option<usize> {
    track_data.iter().position(|x| x.selected)
}

// Helper function for recursively collecting the indices of selected channels
fn select_channel(channel_data: &Vec<ChannelData>, index: usize, selected: &mut Vec<usize>) {
    if let Some(data) = channel_data.get(index) {
        selected.push(index);
        for subchannel in data.subchannels.iter() {
            select_channel(channel_data, *subchannel, selected);
        }
    }
}

#[derive(Debug, Clone, Lens, Data)]
pub struct ChannelData {
    pub name: String,
    pub color: Color,
    pub selected: bool,
    pub subchannels: Vec<usize>,
}

impl Model for ChannelData {}

#[derive(Debug, Clone, Lens, Data)]
pub struct PatternData {
    pub name: String,
    pub channel: usize,
}

impl Model for PatternData {}

#[derive(Debug, Clone, Lens, Data)]
pub struct TrackData {
    pub name: String,
    pub color: Color,
    pub selected: bool,
    // pub audio_clips: Vec<AudioClipData>,
}

impl Model for TrackData {}
