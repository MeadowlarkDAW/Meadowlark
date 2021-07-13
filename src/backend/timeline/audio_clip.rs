use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{MusicalTime, Seconds};
use std::path::PathBuf;

use crate::backend::generic_nodes::{DB_GRADIENT, SMOOTH_MS};
use crate::backend::parameter::{ParamF32, ParamF32Handle, Unit};
use crate::backend::pcm::{AnyPcm, PcmLoadError, PcmLoader};

pub static AUDIO_CLIP_GAIN_MIN_DB: f32 = -40.0;
pub static AUDIO_CLIP_GAIN_MAX_DB: f32 = 40.0;

#[derive(Debug, Clone)]
pub struct AudioClipSaveState {
    pub id: String,
    pub pcm_path: PathBuf,

    pub timeline_start: MusicalTime,
    pub timeline_duration: MusicalTime,

    pub clip_start_offset: Seconds,
    pub clip_gain_db: f32,
}

pub struct AudioClipParamsHandle {
    pub clip_gain_db: ParamF32Handle,

    clip_start_offset: Shared<SharedCell<Seconds>>,
    coll_handle: Handle,
}

impl AudioClipParamsHandle {
    pub fn clip_start_offset(&self) -> Seconds {
        Seconds::clone(&self.clip_start_offset.get())
    }

    pub fn set_clip_start_offset(&mut self, clip_start_offset: Seconds) {
        self.clip_start_offset
            .set(Shared::new(&self.coll_handle, clip_start_offset));
    }
}

pub struct AudioClipParams {
    pub clip_gain_amp: ParamF32,
    pub clip_start_offset: Shared<SharedCell<Seconds>>,
}

impl AudioClipParams {
    fn new(
        clip_gain_db: f32,
        clip_start_offset: Seconds,
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, AudioClipParamsHandle) {
        let clip_gain_db = clip_gain_db.clamp(AUDIO_CLIP_GAIN_MIN_DB, AUDIO_CLIP_GAIN_MAX_DB);

        let (gain_amp, gain_handle) = ParamF32::from_value(
            clip_gain_db,
            AUDIO_CLIP_GAIN_MIN_DB,
            AUDIO_CLIP_GAIN_MAX_DB,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_MS,
            sample_rate,
            coll_handle.clone(),
        );

        let clip_start_offset = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(&coll_handle, clip_start_offset)),
        );

        (
            Self {
                clip_gain_amp: gain_amp,
                clip_start_offset: Shared::clone(&clip_start_offset),
            },
            AudioClipParamsHandle {
                clip_gain_db: gain_handle,
                clip_start_offset,
                coll_handle,
            },
        )
    }
}

#[derive(Clone)]
pub struct AudioClipProcInfo {
    // PcmResources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    pub pcm: Shared<AnyPcm>,

    // Using AtomicRefCell here so we can clone this struct when recompiling the timeline track
    // process. This should never cause a panic because this will only ever be dereferenced once by
    // the rt thread during the current process cycle.
    pub params: Shared<AtomicRefCell<AudioClipParams>>,
}

impl AudioClipProcInfo {
    pub fn new(
        save_state: &AudioClipSaveState,
        pcm_loader: &mut PcmLoader,
        sample_rate: f32,
        coll_handle: &Handle,
    ) -> (Self, AudioClipParamsHandle, Result<(), PcmLoadError>) {
        let (pcm, res) = pcm_loader.load(&save_state.pcm_path);

        let (params, params_handle) = AudioClipParams::new(
            save_state.clip_gain_db,
            save_state.clip_start_offset,
            sample_rate,
            coll_handle.clone(),
        );

        (
            AudioClipProcInfo {
                pcm,
                params: Shared::new(coll_handle, AtomicRefCell::new(params)),
            },
            params_handle,
            res,
        )
    }
}
