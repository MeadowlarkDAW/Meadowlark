// TODO: Eventually this should be moved into the `rusty-daw-timeline` repo.

mod state;
mod tempo_map;

pub mod audio_clip;
pub mod timeline_track_node;
pub mod transport;

pub use audio_clip::{
    AudioClipFades, AudioClipHandle, AudioClipProcess, AudioClipResource, AudioClipResourceCache,
};
pub use state::{AudioClipState, TimelineTrackState, TimelineTransportState};
pub use tempo_map::TempoMap;
pub use timeline_track_node::{TimelineGlobalData, TimelineTrackHandle, TimelineTrackNode};
pub use transport::{LoopState, TimelineTransport, TimelineTransportHandle};
