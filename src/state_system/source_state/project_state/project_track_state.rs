use dropseed::plugin_api::decibel::db_to_coeff_f32;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{
    audio_clip_renderer::AudioClipRenderer,
    resource_loader::{PcmKey, ResourceLoader},
    timeline_track_plug::TimelineTrackPlugState,
};
use crate::state_system::time::{SuperclockTime, TempoMap, Timestamp};

use super::PaletteColor;

static MAX_CROSSFADE_SECONDS: u32 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Synth,
    //Folder, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackRouteType {
    ToMaster,
    ToTrackAtIndex(usize),
    None,
}

#[derive(Debug, Clone)]
pub struct ProjectTrackState {
    pub name: String,
    pub color: PaletteColor,
    pub lane_height: f32,
    pub type_: TrackType,
    pub volume_normalized: f32,
    pub pan_normalized: f32,

    pub routed_to: TrackRouteType,
    //pub parent_track_index: Option<usize>, // TODO
    pub clips: Vec<Rc<RefCell<ClipState>>>,
}

impl ProjectTrackState {
    pub fn into_timeline_track_plug_state(
        &self,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) -> TimelineTrackPlugState {
        let mut audio_clip_renderers: Vec<AudioClipRenderer> = Vec::with_capacity(self.clips.len());

        for clip_state in self.clips.iter() {
            let clip_state = clip_state.borrow();

            let timeline_start = match clip_state.timeline_start {
                Timestamp::Musical(t) => tempo_map.musical_to_nearest_frame_round(t),
                Timestamp::Superclock(t) => t.to_nearest_frame_round(tempo_map.sample_rate()),
            };

            match &clip_state.type_ {
                ClipType::Audio(audio_clip_state) => {
                    let (pcm, _result) = resource_loader.load_pcm(&audio_clip_state.pcm_key);

                    let timeline_end = timeline_start
                        + audio_clip_state
                            .clip_length
                            .to_nearest_frame_round(tempo_map.sample_rate());

                    let mut clip_to_pcm_offset = audio_clip_state
                        .clip_to_pcm_offset
                        .to_nearest_frame_round(tempo_map.sample_rate())
                        .0 as i64;
                    if audio_clip_state.clip_to_pcm_offset_is_negative {
                        clip_to_pcm_offset *= -1;
                    }

                    let incrossfade_len = audio_clip_state
                        .incrossfade_time
                        .to_nearest_frame_round(tempo_map.sample_rate())
                        .0 as u32;
                    let outcrossfade_len = audio_clip_state
                        .outcrossfade_time
                        .to_nearest_frame_round(tempo_map.sample_rate())
                        .0 as u32;

                    let incrossfade_len_recip =
                        if incrossfade_len == 0 { 0.0 } else { 1.0 / incrossfade_len as f64 };
                    let outcrossfade_len_recip =
                        if outcrossfade_len == 0 { 0.0 } else { 1.0 / outcrossfade_len as f64 };

                    let gain_amplitude = db_to_coeff_f32(audio_clip_state.gain_db);

                    audio_clip_renderers.push(AudioClipRenderer {
                        pcm,
                        timeline_start,
                        timeline_end,
                        clip_to_pcm_offset,
                        clip_length: timeline_end - timeline_start,
                        gain_amplitude,
                        incrossfade_type: audio_clip_state.incrossfade_type,
                        incrossfade_len,
                        incrossfade_len_recip,
                        outcrossfade_type: audio_clip_state.outcrossfade_type,
                        outcrossfade_len,
                        outcrossfade_len_recip,
                    });
                }
            }
        }

        TimelineTrackPlugState { audio_clip_renderers }
    }
}

#[derive(Debug, Clone)]
pub struct ClipState {
    pub timeline_start: Timestamp,
    pub name: String,
    pub type_: ClipType,
}

#[derive(Debug, Clone)]
pub enum ClipType {
    Audio(AudioClipState),
}

#[derive(Debug, Clone)]
pub struct AudioClipState {
    pub clip_length: SuperclockTime,

    pub pcm_key: PcmKey,

    // TODO: Automated gain.
    pub gain_db: f32,

    pub clip_to_pcm_offset: SuperclockTime,
    pub clip_to_pcm_offset_is_negative: bool,

    pub incrossfade_type: CrossfadeType,
    pub incrossfade_time: SuperclockTime,

    pub outcrossfade_type: CrossfadeType,
    pub outcrossfade_time: SuperclockTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossfadeType {
    ConstantPower,
    Linear,
    //Symmetric, // TODO
    //Fast, // TODO
    //Slow, // TODO
}

impl Default for CrossfadeType {
    fn default() -> Self {
        CrossfadeType::ConstantPower
    }
}
