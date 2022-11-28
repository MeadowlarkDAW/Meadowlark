use super::PaletteColor;

pub static DEFAULT_TRACK_LANE_HEIGHT: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Synth,
    //Folder, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackRouteType {
    ToMaster,
    ToTrackWithIndex(usize),
    None,
}

#[derive(Debug, Clone)]
pub struct TrackState {
    pub name: String,
    pub color: PaletteColor,
    pub lane_height: f32,
    pub type_: TrackType,

    pub routed_to: TrackRouteType,
    //pub parent_track_index: Option<usize>, // TODO
}

#[derive(Debug, Clone)]
pub struct TracksState {
    pub master_track_color: PaletteColor,
    pub master_track_lane_height: f32,

    pub tracks: Vec<TrackState>,
}

impl TracksState {
    pub fn new() -> Self {
        Self {
            master_track_color: PaletteColor::Unassigned,
            master_track_lane_height: DEFAULT_TRACK_LANE_HEIGHT,

            tracks: vec![
                TrackState {
                    name: "Spicy Synth".into(),
                    color: PaletteColor::Color0,
                    lane_height: DEFAULT_TRACK_LANE_HEIGHT,
                    type_: TrackType::Synth,
                    routed_to: TrackRouteType::ToMaster,
                },
                TrackState {
                    name: "Drum Hits".into(),
                    color: PaletteColor::Color1,
                    lane_height: DEFAULT_TRACK_LANE_HEIGHT,
                    type_: TrackType::Audio,
                    routed_to: TrackRouteType::ToMaster,
                },
            ],
        }
    }
}
