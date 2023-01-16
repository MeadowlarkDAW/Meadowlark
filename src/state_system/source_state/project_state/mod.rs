use crate::state_system::time::{MusicalTime, SuperclockTime, TempoMap, Timestamp};
use crate::{
    resource::PcmKey, ui::panels::timeline_panel::track_header_view::DEFAULT_TRACK_HEADER_HEIGHT,
};

pub mod palette;
pub mod project_track_state;

pub use palette::PaletteColor;
use pcm_loader::ResampleQuality;
pub use project_track_state::{
    AudioClipCopyableState, AudioClipState, CrossfadeType, ProjectAudioTrackState,
    ProjectTrackState, TrackRouteType, TrackType,
};

pub static DEFAULT_TIMELINE_ZOOM: f64 = 0.25;

/// This struct contains all of the state in a given project which can
/// be considered the "source of truth". All other state is derived from
/// the project state.
///
/// This project state is also what gets turned into a "save file".
///
/// This is only allowed to be mutated within the `state_system::handle_action` method.
#[derive(Debug, Clone)]
pub struct ProjectState {
    pub master_track_color: PaletteColor,
    pub master_track_lane_height: f32,
    pub master_track_volume_normalized: f32,
    pub master_track_pan_normalized: f32,

    pub tracks: Vec<ProjectTrackState>,

    /// The horizontal zoom level. 0.25 = default zoom
    pub timeline_horizontal_zoom: f64,

    pub timeline_scroll_beats_x: f64,

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
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                    type_: TrackType::Audio(ProjectAudioTrackState {
                        clips: vec![AudioClipState {
                            name: "Spicy Synth #1".into(),
                            pcm_key: PcmKey {
                                path: "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav"
                                    .into(),
                                resample_to_project_sr: true,
                                resample_quality: ResampleQuality::default(),
                            },
                            copyable: AudioClipCopyableState {
                                timeline_start: Timestamp::Musical(MusicalTime::from_beats(1)),
                                clip_length: SuperclockTime::from_seconds_f64(4.0.into()),
                                gain_db: 0.0,
                                clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                clip_to_pcm_offset_is_negative: false,
                                incrossfade_type: CrossfadeType::Linear,
                                incrossfade_time: SuperclockTime::new(0, 0),
                                outcrossfade_type: CrossfadeType::Linear,
                                outcrossfade_time: SuperclockTime::new(0, 0),
                            },
                        }],
                    }),
                },
                ProjectTrackState {
                    name: "Drum Hits".into(),
                    color: PaletteColor::Color1,
                    lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                    type_: TrackType::Audio(ProjectAudioTrackState {
                        clips: vec![
                            AudioClipState {
                                name: "Drum Loop #1".into(),
                                pcm_key: PcmKey {
                                    path: "./assets/test_files/drums/kick.wav".into(),
                                    resample_to_project_sr: true,
                                    resample_quality: ResampleQuality::default(),
                                },
                                copyable: AudioClipCopyableState {
                                    timeline_start: Timestamp::Musical(MusicalTime::from_beats(2)),
                                    clip_length: SuperclockTime::from_seconds_f64(2.0.into()),
                                    gain_db: 0.0,
                                    clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                    clip_to_pcm_offset_is_negative: false,
                                    incrossfade_type: CrossfadeType::Linear,
                                    incrossfade_time: SuperclockTime::new(0, 0),
                                    outcrossfade_type: CrossfadeType::Linear,
                                    outcrossfade_time: SuperclockTime::new(0, 0),
                                },
                            },
                            AudioClipState {
                                name: "Drum Loop #2".into(),
                                pcm_key: PcmKey {
                                    path: "./assets/test_files/drums/snare.wav".into(),
                                    resample_to_project_sr: true,
                                    resample_quality: ResampleQuality::default(),
                                },
                                copyable: AudioClipCopyableState {
                                    timeline_start: Timestamp::Musical(
                                        MusicalTime::from_quarter_beats(8, 1),
                                    ),
                                    clip_length: SuperclockTime::from_seconds_f64(8.0.into()),
                                    gain_db: 0.0,
                                    clip_to_pcm_offset: SuperclockTime::new(0, 0),
                                    clip_to_pcm_offset_is_negative: false,
                                    incrossfade_type: CrossfadeType::Linear,
                                    incrossfade_time: SuperclockTime::new(0, 0),
                                    outcrossfade_type: CrossfadeType::Linear,
                                    outcrossfade_time: SuperclockTime::new(0, 0),
                                },
                            },
                        ],
                    }),
                },
            ],

            timeline_horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            timeline_scroll_beats_x: 0.0,

            loop_start: Timestamp::Musical(MusicalTime::from_beats(8)),
            loop_end: Timestamp::Musical(MusicalTime::from_beats(16)),
            loop_active: true,

            playhead_last_seeked: Timestamp::Musical(MusicalTime::from_beats(0)),

            tempo_map: TempoMap::default(),
        }
    }
}
