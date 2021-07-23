use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use fnv::FnvHashMap;
use rusty_daw_time::TempoMap;
use std::sync::{Arc, Mutex};

use crate::backend::graph_interface::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer,
};
use crate::backend::resource_loader::{PcmLoadError, ResourceLoadError, ResourceLoader};

pub mod audio_clip;
pub use audio_clip::AudioClipSaveState;

pub mod transport;
pub use transport::{
    LoopStatus, TimelineTransport, TimelineTransportHandle, TimelineTransportSaveState,
    TransportStatus,
};

use audio_clip::{AudioClipHandle, AudioClipProcess};

#[derive(Debug)]
pub struct TimelineTrackSaveState {
    /// The ID (name) of the timeline track. This must be unique for
    /// each timeline track.
    pub id: String,

    /// The audio clips in this track.
    pub audio_clips: Vec<AudioClipSaveState>,
}

pub struct TimelineTrackHandle {
    audio_clip_indexes: FnvHashMap<String, usize>,
    audio_clip_handles: Vec<AudioClipHandle>,

    process: Shared<SharedCell<TimelineTrackProcess>>,

    sample_rate: f32,
    coll_handle: Handle,
}

impl TimelineTrackHandle {
    /// Return an immutable handle to the audio clip with given ID.
    pub fn audio_clip<'a>(
        &'a self,
        id: &String,
        save_state: &'a TimelineTrackSaveState,
    ) -> Option<(&'a AudioClipHandle, &'a AudioClipSaveState)> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            Some((
                &self.audio_clip_handles[*audio_clip_index],
                &save_state.audio_clips[*audio_clip_index],
            ))
        } else {
            None
        }
    }

    /// Return a mutable handle to the audio clip with given ID.
    pub fn audio_clip_mut<'a>(
        &'a mut self,
        id: &String,
        save_state: &'a mut TimelineTrackSaveState,
    ) -> Option<(&'a mut AudioClipHandle, &'a mut AudioClipSaveState)> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            Some((
                &mut self.audio_clip_handles[*audio_clip_index],
                &mut save_state.audio_clips[*audio_clip_index],
            ))
        } else {
            None
        }
    }

    /// Set the ID of the audio clip. The audio clip's ID is used as the name. It must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn set_audio_clip_id(
        &mut self,
        old_id: &String,
        new_id: String,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), ()> {
        if self.audio_clip_indexes.contains_key(&new_id) {
            return Err(());
        }

        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(old_id) {
            self.audio_clip_indexes
                .insert(new_id.clone(), audio_clip_index);

            // Update the values in the save state.
            save_state.audio_clips[audio_clip_index].id = new_id;

            Ok(())
        } else {
            Err(())
        }
    }

    /// Add a new audio clip to this track. The ID must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn add_audio_clip(
        &mut self,
        clip: AudioClipSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        tempo_map: &TempoMap,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<Result<(), PcmLoadError>, ()> {
        if self.audio_clip_indexes.contains_key(&clip.id) {
            return Err(());
        }

        let audio_clip_index = save_state.audio_clips.len();
        self.audio_clip_indexes
            .insert(clip.id.clone(), audio_clip_index);

        let (audio_clip_proc, params_handle, res) =
            AudioClipProcess::new(&clip, resource_loader, tempo_map, self.coll_handle.clone());

        // Compile the new process.

        let mut new_process = TimelineTrackProcess::clone(&self.process.get());

        // Clone the old processes.
        let mut new_audio_clip_procs = Vec::clone(&new_process.audio_clips);

        // Add the new clip.
        new_audio_clip_procs.push(audio_clip_proc);

        // Use the new process info.
        new_process.audio_clips = Shared::new(&self.coll_handle, new_audio_clip_procs);
        self.process
            .set(Shared::new(&self.coll_handle, new_process));

        self.audio_clip_handles.push(params_handle);
        save_state.audio_clips.push(clip);

        Ok(res)
    }

    /// Remove an audio clip from this track.
    pub fn remove_audio_clip(
        &mut self,
        id: &String,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(id) {
            save_state.audio_clips.remove(audio_clip_index);
            self.audio_clip_handles.remove(audio_clip_index);

            // Shift every clip's index that appears after this one.
            for (_, clip_index) in self.audio_clip_indexes.iter_mut() {
                if *clip_index > audio_clip_index {
                    *clip_index -= 1;
                }
            }

            // Compile the new process.

            let mut new_process = TimelineTrackProcess::clone(&self.process.get());

            // Clone the old processes.
            let mut new_audio_clip_procs = Vec::clone(&new_process.audio_clips);

            // Remove the old clip.
            new_audio_clip_procs.remove(audio_clip_index);

            // Use the new processes.
            new_process.audio_clips = Shared::new(&self.coll_handle, new_audio_clip_procs);
            self.process
                .set(Shared::new(&self.coll_handle, new_process));

            Ok(())
        } else {
            Err(())
        }
    }

    pub(super) fn update_tempo_map(
        &mut self,
        tempo_map: &TempoMap,
        save_state: &TimelineTrackSaveState,
    ) {
        for (clip, save) in self
            .audio_clip_handles
            .iter_mut()
            .zip(save_state.audio_clips.iter())
        {
            clip.update_tempo_map(tempo_map, save);
        }
    }
}

pub struct TimelineTrackNode {
    sample_rate: f32,

    process: Shared<SharedCell<TimelineTrackProcess>>,
}

impl TimelineTrackNode {
    pub fn new(
        save_state: &TimelineTrackSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        tempo_map: &TempoMap,
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, TimelineTrackHandle, Vec<ResourceLoadError>) {
        let mut audio_clip_procs = Vec::<AudioClipProcess>::new();
        let mut audio_clip_errors = Vec::<ResourceLoadError>::new();
        let mut audio_clip_indexes = FnvHashMap::<String, usize>::default();
        let mut audio_clip_handles = Vec::<AudioClipHandle>::new();

        for (audio_clip_index, audio_clip_save) in save_state.audio_clips.iter().enumerate() {
            let (process, handle, res) = AudioClipProcess::new(
                audio_clip_save,
                resource_loader,
                tempo_map,
                coll_handle.clone(),
            );

            if let Err(e) = res {
                audio_clip_errors.push(ResourceLoadError::PCM(e));
            }

            audio_clip_procs.push(process);
            audio_clip_indexes.insert(audio_clip_save.id.clone(), audio_clip_index);
            audio_clip_handles.push(handle);
        }

        let process = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                TimelineTrackProcess {
                    audio_clips: Shared::new(&coll_handle, audio_clip_procs),
                },
            )),
        );

        (
            Self {
                sample_rate,
                process: Shared::clone(&process),
            },
            TimelineTrackHandle {
                audio_clip_indexes,
                audio_clip_handles,
                process,
                sample_rate,
                coll_handle,
            },
            audio_clip_errors,
        )
    }
}

impl AudioGraphNode for TimelineTrackNode {
    // TODO: Switch between mono and stereo based on the type of audio
    // clips on the track.
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        transport: &TimelineTransport,
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let process = self.process.get();

        for audio_clip in process.audio_clips.iter() {
            audio_clip.process(proc_info, transport, stereo_audio_out)
        }
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess {
    audio_clips: Shared<Vec<AudioClipProcess>>,
}
