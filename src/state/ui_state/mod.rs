use rusty_daw_core::{MusicalTime, SampleRate, SuperFrames};
use std::path::PathBuf;
use vizia::{Context, Data, Event, Lens, Model};

use crate::backend::timeline::{AudioClipState, LoopState};

/// This struct should contain all state that the UI will bind to. This should
/// mirror the `ProjectState` plus whatever extra state is needed for the UI.
///
/// (Yes we are duplicating state from `ProjectState`). This is for a couple
/// of reasons:
///
/// 1. This separates areas of concerns, so the UI can be developed independently
/// of the backend.
/// 2. Even if a project is not loaded in the backend, the UI should still show
/// something in its place like empty tracks and mixers.
/// 3. This makes it clearer what state the GUI cares about by consolidating all
/// state into the `ui_state` folder (as apposed to state being scattered around
/// the backend and various other 3rd party crates).
/// 4. This will make it easier to create "bindings/lenses" for data-driven UI
/// paridigms.
/// 5. This `UiState` struct is only exposed to the UI as an immutable reference
/// via the `StateSystem` struct. This ensures that any mutation of state *must*
/// go through the `StateSystem` struct which is responsible for keeping
/// everything in sync.
/// 6. Memory is cheap nowadays anyway, and it's not like we're cloning large
/// blocks of data like audio samples (the largest things we will clone will
/// mostly just be strings, piano roll clips, and automation tracks).
#[derive(Lens)]
pub struct UiState {
    pub backend_loaded: bool,

    pub timeline_transport: TimelineTransportUiState,
    pub tempo_map: TempoMapUiState,
    pub sample_rate: SampleRate,

    pub timeline_tracks: Vec<TimelineTrackUiState>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            backend_loaded: false,

            timeline_transport: TimelineTransportUiState::default(),
            tempo_map: TempoMapUiState { bpm: 110.0 },

            timeline_tracks: Vec::new(),
            sample_rate: SampleRate::default(),
        }
    }
}

impl Model for UiState {}

// State which describes a selection within the timeline view
#[derive(Debug, Clone, Copy, Data, Lens)]
pub struct TimelineSelectionUiState {
    pub track_start: usize,
    pub track_end: usize,
    pub select_start: MusicalTime,
    pub select_end: MusicalTime,

    pub hovered_track: usize,
}

#[derive(Debug)]
pub enum TimelineSelectionEvent {
    SetHoveredTrack(usize),
    // track_start, track_end, select_start, select_end
    SetSelection(usize, usize, MusicalTime, MusicalTime),

    SelectNone,
}

impl Model for TimelineSelectionUiState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(timeline_selection_event) = event.message.downcast() {
            match timeline_selection_event {
                TimelineSelectionEvent::SetHoveredTrack(track_id) => {
                    //println!("Hovered Track: {}", track_id);
                    self.hovered_track = *track_id;
                }

                TimelineSelectionEvent::SetSelection(
                    track_start,
                    track_end,
                    select_start,
                    select_end,
                ) => {
                    //println!("track_start: {}, track_end: {}, select_start: {:?}, select_end: {:?}", track_start, track_end, select_start, select_end);
                    self.track_start = *track_start;
                    self.track_end = *track_end;
                    self.select_start = *select_start;
                    self.select_end = *select_end;
                }

                TimelineSelectionEvent::SelectNone => {
                    self.track_start = 0;
                    self.track_end = 0;
                    self.select_start = MusicalTime::new(0, 0);
                    self.select_end = MusicalTime::new(0, 0);
                }
            }
        }
    }
}

#[derive(Lens)]
pub struct TempoMapUiState {
    // TODO: This will need to change once we start to support automation of tempo.
    pub bpm: f64,
}

impl Model for TempoMapUiState {}

#[derive(Clone, Data, Lens)]
pub struct TimelineTransportUiState {
    pub is_playing: bool,
    /// The place where the playhead will seek to on project load/transport stop.
    pub seek_to: MusicalTime,
    pub loop_state: LoopUiState,
    pub playhead: MusicalTime,
}

impl Model for TimelineTransportUiState {}

impl Default for TimelineTransportUiState {
    fn default() -> Self {
        Self {
            is_playing: false,
            seek_to: MusicalTime::new(0, 0),
            loop_state: LoopUiState {
                status: LoopStatusUiState::Inactive,
                loop_start: MusicalTime::new(0, 0),
                loop_end: MusicalTime::new(8, 0),
            },
            playhead: MusicalTime::new(0, 0),
        }
    }
}

/// The status of looping on this transport.
#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum LoopStatusUiState {
    /// The transport is not currently looping.
    Inactive,
    /// The transport is currently looping.
    Active,
}

/// The status of looping on this transport.
#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub struct LoopUiState {
    pub status: LoopStatusUiState,
    /// The start of the loop (inclusive).
    pub loop_start: MusicalTime,
    /// The end of the loop (exclusive).
    pub loop_end: MusicalTime,
}

impl From<LoopState> for LoopUiState {
    fn from(l: LoopState) -> Self {
        match l {
            LoopState::Inactive => LoopUiState {
                status: LoopStatusUiState::Inactive,
                loop_start: MusicalTime::new(0, 0),
                loop_end: MusicalTime::new(8, 0),
            },
            LoopState::Active { loop_start, loop_end } => {
                LoopUiState { status: LoopStatusUiState::Active, loop_start, loop_end }
            }
        }
    }
}

impl LoopUiState {
    pub fn to_backend_state(&self) -> LoopState {
        match self.status {
            LoopStatusUiState::Inactive => LoopState::Inactive,
            LoopStatusUiState::Active => {
                LoopState::Active { loop_start: self.loop_start, loop_end: self.loop_end }
            }
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct TimelineTrackUiState {
    /// The name displayed on this timeline track.
    pub name: String,

    /// The height of the timeline track in pixels
    pub height: f32,

    /// The audio clips on this timeline track. These may not be
    /// in any particular order.
    pub audio_clips: Vec<AudioClipUiState>,
}

impl Model for TimelineTrackUiState {}

#[derive(Debug, Clone, Data, Lens)]
pub struct AudioClipUiState {
    /// The name displayed on the audio clip.
    pub name: String,

    /// The path to the audio file containing the PCM data.
    #[data(ignore)]
    pub pcm_path: PathBuf,

    /// Where the clip starts on the timeline.
    pub timeline_start: MusicalTime,

    /// The duration of the clip on the timeline.
    pub duration: SuperFrames,

    /// The offset in the pcm resource where the "start" of the clip should start playing from.
    pub clip_start_offset: SuperFrames,

    pub clip_start_offset_is_negative: bool,

    /// The gain of the audio clip in decibels.
    pub clip_gain_db: f32,

    /// The fades on this audio clip.
    pub fades: AudioClipFadesUiState,
}

impl Model for AudioClipUiState {}

impl From<&AudioClipState> for AudioClipUiState {
    fn from(a: &AudioClipState) -> Self {
        Self {
            name: a.name.clone(),
            pcm_path: a.pcm_path.clone(),
            timeline_start: a.timeline_start,
            duration: a.duration,
            clip_start_offset: a.clip_start_offset,
            clip_start_offset_is_negative: a.clip_start_offset_is_negative,
            clip_gain_db: a.clip_gain_db,
            fades: AudioClipFadesUiState {
                start_fade_duration: a.fades.start_fade_duration,
                end_fade_duration: a.fades.end_fade_duration,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Data, Lens)]
pub struct AudioClipFadesUiState {
    pub start_fade_duration: SuperFrames,
    pub end_fade_duration: SuperFrames,
}

impl Model for AudioClipFadesUiState {}
