use vizia::prelude::*;

use super::app_event::AppEvent;

// TODO - Move this somewhere probably
#[derive(Lens)]
pub struct AppData {
    pub channel_data: Vec<ChannelData>,
    pub pattern_data: Vec<PatternData>,
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
        });
    }
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
