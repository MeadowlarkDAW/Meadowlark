use crate::state_system::time::{MusicalTime, SuperclockTime, TempoMap, Timestamp};
use crate::{
    backend::resource_loader::PcmKey, state_system::ScrollUnits,
    ui::panels::timeline_panel::track_header_view::DEFAULT_TRACK_HEADER_HEIGHT,
};

pub mod palette;
pub mod project_track_state;

pub use palette::PaletteColor;
use pcm_loader::ResampleQuality;
pub use project_track_state::{ProjectTrackState, TrackRouteType, TrackType};

use self::project_track_state::{AudioClipState, ClipState, ClipType, CrossfadeType};

pub static DEFAULT_TIMELINE_ZOOM: f64 = 0.25;

/// This struct contains all of the state in a given project which can
/// be considered the "source of truth". All other state is derived from
/// the project state.
///
/// This project state is also what gets turned into a "save file".
///
/// Only the `StateSystem` struct is allowed to mutate this.
#[derive(Debug, Clone)]
pub struct ProjectState {
    pub master_track_color: PaletteColor,
    pub master_track_lane_height: f32,
    pub master_track_volume_normalized: f32,
    pub master_track_pan_normalized: f32,

    pub tracks: Vec<ProjectTrackState>,

    /// The horizontal zoom level. 0.25 = default zoom
    pub timeline_horizontal_zoom: f64,

    pub timeline_scroll_units_x: ScrollUnits,

    /// The mode in which the timeline displays its contents.
    pub timeline_mode: TimelineMode,

    pub loop_start: Timestamp,
    pub loop_end: Timestamp,
    pub loop_active: bool,

    pub playhead_last_seeked: Timestamp,

    pub tempo_map: TempoMap,
}

impl ProjectState {
    pub fn test_project() -> Self {
        Self {
            master_track_color: PaletteColor::Unassigned,
            master_track_lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
            master_track_volume_normalized: 1.0,
            master_track_pan_normalized: 0.5,

            tracks: vec![
                ProjectTrackState {
                    name: "Spicy Synth".into(),
                    color: PaletteColor::Color0,
                    lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: TrackType::Synth,
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                    clips: [(
                        0,
                        ClipState {
                            timeline_start: Timestamp::Musical(MusicalTime::from_beats(1)),
                            name: "Spicy Synth #1".into(),
                            type_: ClipType::Audio(AudioClipState {
                                clip_length: SuperclockTime::from_seconds_f64(4.0.into()),
                                pcm_key: PcmKey {
                                    path:
                                        "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav"
                                            .into(),
                                    resample_to_project_sr: true,
                                    resample_quality: ResampleQuality::default(),
                                },
                                gain_db: 0.0,
                                clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                clip_to_pcm_offset_is_negative: false,
                                incrossfade_type: CrossfadeType::Linear,
                                incrossfade_time: SuperclockTime::new(0, 0),
                                outcrossfade_type: CrossfadeType::Linear,
                                outcrossfade_time: SuperclockTime::new(0, 0),
                            }),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
                ProjectTrackState {
                    name: "Drum Hits".into(),
                    color: PaletteColor::Color1,
                    lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: TrackType::Audio,
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                    clips: [
                        (
                            0,
                            ClipState {
                                timeline_start: Timestamp::Musical(MusicalTime::from_beats(2)),
                                name: "Drum Loop #1".into(),
                                type_: ClipType::Audio(AudioClipState {
                                    clip_length: SuperclockTime::from_seconds_f64(2.0.into()),
                                    pcm_key: PcmKey {
                                        path: "./assets/test_files/drums/kick.wav".into(),
                                        resample_to_project_sr: true,
                                        resample_quality: ResampleQuality::default(),
                                    },
                                    gain_db: 0.0,
                                    clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                    clip_to_pcm_offset_is_negative: false,
                                    incrossfade_type: CrossfadeType::Linear,
                                    incrossfade_time: SuperclockTime::new(0, 0),
                                    outcrossfade_type: CrossfadeType::Linear,
                                    outcrossfade_time: SuperclockTime::new(0, 0),
                                }),
                            },
                        ),
                        (
                            1,
                            ClipState {
                                timeline_start: Timestamp::Musical(
                                    MusicalTime::from_quarter_beats(8, 1),
                                ),
                                name: "Drum Loop #2".into(),
                                type_: ClipType::Audio(AudioClipState {
                                    clip_length: SuperclockTime::from_seconds_f64(8.0.into()),
                                    pcm_key: PcmKey {
                                        path: "./assets/test_files/drums/snare.wav".into(),
                                        resample_to_project_sr: true,
                                        resample_quality: ResampleQuality::default(),
                                    },
                                    gain_db: 0.0,
                                    clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                    clip_to_pcm_offset_is_negative: false,
                                    incrossfade_type: CrossfadeType::Linear,
                                    incrossfade_time: SuperclockTime::new(0, 0),
                                    outcrossfade_type: CrossfadeType::Linear,
                                    outcrossfade_time: SuperclockTime::new(0, 0),
                                }),
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            ],

            timeline_horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            timeline_scroll_units_x: ScrollUnits::Musical(0.0),
            timeline_mode: TimelineMode::Musical,

            loop_start: Timestamp::Musical(MusicalTime::from_beats(8)),
            loop_end: Timestamp::Musical(MusicalTime::from_beats(16)),
            loop_active: true,

            playhead_last_seeked: Timestamp::Musical(MusicalTime::from_beats(0)),

            tempo_map: TempoMap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// In this mode, the timeline displays content in units of measures,
    /// bars, beats, and sub-beats.
    Musical,
    /// In this mode, the timeline displays content in units of hours,
    /// minutes, seconds, and milliseconds.
    HMS,
}
