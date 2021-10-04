// TODO: Eventually this should be moved into the `rusty-daw-timeline` repo.

mod tempo_map;

pub mod audio_clip;
pub mod timeline_track_node;
pub mod transport;

pub use audio_clip::{
    AudioClipHandle, AudioClipProcess, AudioClipResource, AudioClipResourceCache,
    AudioClipSaveState,
};
pub use tempo_map::TempoMap;
pub use timeline_track_node::{TimelineTrackHandle, TimelineTrackNode};
pub use transport::{
    LoopState, TimelineTransport, TimelineTransportHandle, TimelineTransportSaveState,
};

#[derive(Debug, Clone)]
pub struct TimelineTrackSaveState {
    name: String,
    audio_clips: Vec<AudioClipSaveState>,
}

impl TimelineTrackSaveState {
    /// Create a new timeline track save state.
    ///
    /// * `name` - The name displayed on this timeline track.
    /// * `audio_clips` - The audio clips on this track. These
    /// do not need to be in any particular order.
    pub fn new(name: String, audio_clips: Vec<AudioClipSaveState>) -> Self {
        Self { name, audio_clips }
    }

    /// The name displayed on this timeline track.
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// The audio clips on this timeline track. These may not be
    /// in any particular order.
    #[inline]
    pub fn audio_clips(&self) -> &[AudioClipSaveState] {
        self.audio_clips.as_slice()
    }
}
