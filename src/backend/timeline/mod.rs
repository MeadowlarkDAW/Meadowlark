use std::path::PathBuf;

use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use fnv::FnvHashMap;
use rusty_daw_time::{MusicalTime, SampleTime, Seconds, TempoMap};

use crate::backend::graph_state::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer, MAX_BLOCKSIZE,
};

use crate::backend::pcm::{PcmLoadError, PcmLoader};

pub mod audio_clip;
pub use audio_clip::AudioClipSaveState;

use audio_clip::{AudioClipParams, AudioClipParamsHandle, AudioClipProcInfo};

pub struct TimelineTrackSaveState {
    name: String,

    audio_clips: Vec<AudioClipSaveState>,
}

pub struct TimelineTrackHandle {
    save_state: TimelineTrackSaveState,

    audio_clip_indexes: FnvHashMap<String, usize>,
    audio_clip_params: Vec<AudioClipParamsHandle>,

    process: Shared<SharedCell<TimelineTrackProcess>>,

    sample_rate: f32,
    coll_handle: Handle,
}

impl TimelineTrackHandle {
    pub fn save_state(&self) -> &TimelineTrackSaveState {
        &self.save_state
    }

    pub fn audio_clip_params(&self, id: &String) -> Option<&AudioClipParamsHandle> {
        self.audio_clip_indexes
            .get(id)
            .map(|i| &self.audio_clip_params[*i])
    }

    pub fn set_audio_clip_gain(&mut self, clip_gain_db: f32, id: &String) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            self.audio_clip_params[*audio_clip_index]
                .clip_gain_db
                .set_value(clip_gain_db);

            // Make sure the value stays in bounds.
            let clip_gain_db = self.audio_clip_params[*audio_clip_index]
                .clip_gain_db
                .value();

            // Update the values in the save state.
            self.save_state.audio_clips[*audio_clip_index].clip_gain_db = clip_gain_db;

            // TODO: Alert the GUI of the change.

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_audio_clip_start_offset(
        &mut self,
        clip_start_offset: Seconds,
        id: &String,
    ) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            self.audio_clip_params[*audio_clip_index].set_clip_start_offset(clip_start_offset);

            // Update the values in the save state.
            self.save_state.audio_clips[*audio_clip_index].clip_start_offset = clip_start_offset;

            // TODO: Alert the GUI of the change.

            Ok(())
        } else {
            Err(())
        }
    }

    /// The audio clip's name is used as the ID. It must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn set_audio_clip_id(&mut self, old_id: &String, new_id: String) -> Result<(), ()> {
        if self.audio_clip_indexes.contains_key(&new_id) {
            return Err(());
        }

        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(old_id) {
            self.save_state.audio_clips[audio_clip_index].id = new_id.clone();

            self.audio_clip_indexes.insert(new_id, audio_clip_index);

            // TODO: Alert the GUI of the change.

            Ok(())
        } else {
            Err(())
        }
    }

    /// Add a new audio clip. The ID must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn add_audio_clip(
        &mut self,
        clip: AudioClipSaveState,
        pcm_loader: &mut PcmLoader,
        tempo_map: &TempoMap,
    ) -> Result<Result<(), PcmLoadError>, ()> {
        if self.audio_clip_indexes.contains_key(&clip.id) {
            return Err(());
        }

        let audio_clip_index = self.save_state.audio_clips.len();
        self.audio_clip_indexes
            .insert(clip.id.clone(), audio_clip_index);

        let (proc_info, params_handle, res) =
            AudioClipProcInfo::new(&clip, pcm_loader, self.sample_rate, &self.coll_handle);

        // Compile the new process.

        let mut new_process = TimelineTrackProcess::clone(&self.process.get());

        // Clone the old process info.
        let mut new_procs_info = Vec::clone(&new_process.audio_clips);

        // Add the new clip.
        new_procs_info.push(proc_info);

        // Clone the old schedule.
        let mut new_schedule = new_process.schedule.clone_new_version();

        Self::insert_audio_clip_events(&mut new_schedule, &clip, audio_clip_index, tempo_map);

        // Use the new process info and schedule.
        new_process.audio_clips = Shared::new(&self.coll_handle, new_procs_info);
        new_process.schedule = Shared::new(&self.coll_handle, new_schedule);
        self.process
            .set(Shared::new(&self.coll_handle, new_process));

        self.audio_clip_params.push(params_handle);
        self.save_state.audio_clips.push(clip);

        // TODO: Alert the GUI of the change.

        Ok(res)
    }

    pub fn remove_audio_clip(&mut self, id: &String) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(id) {
            self.save_state.audio_clips.remove(audio_clip_index);
            self.audio_clip_params.remove(audio_clip_index);

            // Shift every clip's index that appears after this one.
            for (_, clip_index) in self.audio_clip_indexes.iter_mut() {
                if *clip_index > audio_clip_index {
                    *clip_index -= 1;
                }
            }

            // Compile the new process.

            let mut new_process = TimelineTrackProcess::clone(&self.process.get());

            // Clone the old process info.
            let mut new_procs_info = Vec::clone(&new_process.audio_clips);

            // Remove the old clip.
            new_procs_info.remove(audio_clip_index);

            // Clone the old schedule.
            let mut new_schedule = new_process.schedule.clone_new_version();

            // Remove the old events for this clip.
            //
            // TODO: This could be optimized with a binary search using the clip's sample times.
            new_schedule.schedule.retain(|(_, event)| match event {
                TimelineEvent::AudioClipStart(index) => *index != audio_clip_index,
                TimelineEvent::AudioClipEnd(index) => *index != audio_clip_index,
                _ => true,
            });

            // Shift every clip's index that appears after this one.
            for (_, event) in new_schedule.schedule.iter_mut() {
                match event {
                    TimelineEvent::AudioClipStart(index) => {
                        if *index > audio_clip_index {
                            *index -= 1;
                        }
                    }
                    TimelineEvent::AudioClipEnd(index) => {
                        if *index > audio_clip_index {
                            *index -= 1;
                        }
                    }
                }
            }

            // Use the new process info and schedule.
            new_process.audio_clips = Shared::new(&self.coll_handle, new_procs_info);
            new_process.schedule = Shared::new(&self.coll_handle, new_schedule);
            self.process
                .set(Shared::new(&self.coll_handle, new_process));

            // TODO: Alert the GUI of the change.

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_audio_clip_pcm_path(
        &mut self,
        path: PathBuf,
        id: &String,
        pcm_loader: &mut PcmLoader,
    ) -> Result<Result<(), PcmLoadError>, ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            let clip = &mut self.save_state.audio_clips[*audio_clip_index];

            if clip.pcm_path != path {
                clip.pcm_path = path;

                let (pcm, res) = pcm_loader.load(&clip.pcm_path);

                // Compile the new process.

                let mut new_process = TimelineTrackProcess::clone(&self.process.get());

                // Recompile process info.
                let mut new_procs_info = Vec::clone(&new_process.audio_clips);
                new_procs_info[*audio_clip_index].pcm = pcm;

                new_process.audio_clips = Shared::new(&self.coll_handle, new_procs_info);
                self.process
                    .set(Shared::new(&self.coll_handle, new_process));

                // TODO: Alert the GUI of the change.

                Ok(res)
            } else {
                Ok(Ok(()))
            }
        } else {
            Err(())
        }
    }

    pub fn set_audio_clip_timeline_pos(
        &mut self,
        timeline_start: MusicalTime,
        timeline_duration: MusicalTime,
        id: &String,
        tempo_map: &TempoMap,
    ) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            let clip = &mut self.save_state.audio_clips[*audio_clip_index];

            if clip.timeline_start != timeline_start || clip.timeline_duration != timeline_duration
            {
                clip.timeline_start = timeline_start;
                clip.timeline_duration = timeline_duration;

                // Compile the new process.

                let mut new_process = TimelineTrackProcess::clone(&self.process.get());

                // Clone the old schedule.
                let mut new_schedule = new_process.schedule.clone_new_version();

                // Remove the old events for this clip.
                //
                // TODO: This could be optimized with a binary search using this clip's previous sample times.
                new_schedule.schedule.retain(|(_, event)| match event {
                    TimelineEvent::AudioClipStart(index) => *index != *audio_clip_index,
                    TimelineEvent::AudioClipEnd(index) => *index != *audio_clip_index,
                    _ => true,
                });

                Self::insert_audio_clip_events(
                    &mut new_schedule,
                    &clip,
                    *audio_clip_index,
                    tempo_map,
                );

                // Use the new schedule.
                new_process.schedule = Shared::new(&self.coll_handle, new_schedule);
                self.process
                    .set(Shared::new(&self.coll_handle, new_process));

                // TODO: Alert the GUI of the change.
            }

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn update_tempo_map(&mut self, tempo_map: &TempoMap) {
        // Compile the new process.

        let mut new_process = TimelineTrackProcess::clone(&self.process.get());

        // Clone the old schedule.
        let mut new_schedule = new_process.schedule.clone_new_version();

        // Update all events.
        for (event, _) in new_schedule.schedule.iter_mut() {
            let sample_start = tempo_map.nearest_sample_round(event.timeline_start);
            let sample_end =
                tempo_map.nearest_sample_round(event.timeline_start + event.timeline_duration);

            if event.is_end {
                event.sample_time = sample_end;
            } else {
                event.sample_time = sample_start;
            }
            event.sample_duration = sample_end - sample_start;
        }

        // In theory all events should still be in chronological order.

        // Use the new schedule.
        new_process.schedule = Shared::new(&self.coll_handle, new_schedule);
        self.process
            .set(Shared::new(&self.coll_handle, new_process));
    }

    fn insert_audio_clip_events(
        schedule: &mut TimelineSchedule,
        clip: &AudioClipSaveState,
        audio_clip_index: usize,
        tempo_map: &TempoMap,
    ) {
        let sample_start = tempo_map.nearest_sample_round(clip.timeline_start);
        let sample_end =
            tempo_map.nearest_sample_round(clip.timeline_start + clip.timeline_duration);

        // Create the new timeline events.
        let start_event = (
            Event {
                timeline_start: clip.timeline_start,
                timeline_duration: clip.timeline_duration,
                sample_time: sample_start,
                sample_duration: sample_end - sample_start,
                is_end: false,
            },
            TimelineEvent::AudioClipStart(audio_clip_index),
        );
        let end_event = (
            Event {
                timeline_start: clip.timeline_start,
                timeline_duration: clip.timeline_duration,
                sample_time: sample_end,
                sample_duration: sample_end - sample_start,
                is_end: true,
            },
            TimelineEvent::AudioClipEnd(audio_clip_index),
        );

        // Insert the new events in order.
        //
        // TODO: This could be optimized using binary search.
        let mut found_start = None;
        for (i, event) in schedule.schedule.iter().enumerate() {
            if &start_event.0 <= &event.0 {
                found_start = Some(i);
                break;
            }
        }
        if let Some(start_i) = found_start {
            schedule.schedule.insert(start_i, start_event);
        } else {
            schedule.schedule.push(start_event);
        }

        let mut found_end = None;
        for (i, event) in schedule.schedule.iter().enumerate() {
            if &end_event.0 <= &event.0 {
                found_end = Some(i);
                break;
            }
        }
        if let Some(end_i) = found_end {
            schedule.schedule.insert(end_i, end_event);
        } else {
            schedule.schedule.push(end_event);
        }
    }
}

pub struct TimelineTrackNode {
    sample_rate: f32,

    process: Shared<SharedCell<TimelineTrackProcess>>,
}

impl TimelineTrackNode {
    pub fn new(
        save_state: TimelineTrackSaveState,
        pcm_loader: &mut PcmLoader,
        tempo_map: &TempoMap,
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (
        Self,
        TimelineTrackHandle,
        Vec<(AudioClipSaveState, PcmLoadError)>,
    ) {
        let mut schedule = TimelineSchedule {
            schedule: Vec::new(),
            version: 0,
        };

        let mut audio_clip_errors = Vec::<(AudioClipSaveState, PcmLoadError)>::new();
        let mut audio_clip_proc = Vec::<AudioClipProcInfo>::new();
        let mut audio_clip_indexes = FnvHashMap::<String, usize>::default();
        let mut audio_clip_params = Vec::<AudioClipParamsHandle>::new();

        for (audio_clip_index, audio_clip_save) in save_state.audio_clips.iter().enumerate() {
            let (proc_info, params_handle, res) =
                AudioClipProcInfo::new(audio_clip_save, pcm_loader, sample_rate, &coll_handle);

            if let Err(e) = res {
                audio_clip_errors.push((audio_clip_save.clone(), e));
            }

            audio_clip_proc.push(proc_info);
            audio_clip_indexes.insert(audio_clip_save.id.clone(), audio_clip_index);
            audio_clip_params.push(params_handle);

            // Create two events for the schedule.
            let sample_start = tempo_map.nearest_sample_round(audio_clip_save.timeline_start);
            let sample_end = tempo_map.nearest_sample_round(
                audio_clip_save.timeline_start + audio_clip_save.timeline_duration,
            );
            let start_event = (
                Event {
                    timeline_start: audio_clip_save.timeline_start,
                    timeline_duration: audio_clip_save.timeline_duration,
                    sample_time: sample_start,
                    sample_duration: sample_end - sample_start,
                    is_end: false,
                },
                TimelineEvent::AudioClipStart(audio_clip_index),
            );
            let end_event = (
                Event {
                    timeline_start: audio_clip_save.timeline_start,
                    timeline_duration: audio_clip_save.timeline_duration,
                    sample_time: sample_end,
                    sample_duration: sample_end - sample_start,
                    is_end: true,
                },
                TimelineEvent::AudioClipEnd(audio_clip_index),
            );

            schedule.schedule.push(start_event);
            schedule.schedule.push(end_event);
        }

        // Sort the schedule in order of sample time.
        schedule
            .schedule
            .sort_by(|(event_a, _), (event_b, _)| event_a.partial_cmp(event_b).unwrap());

        let process = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                TimelineTrackProcess {
                    audio_clips: Shared::new(&coll_handle, audio_clip_proc),
                    schedule: Shared::new(&coll_handle, schedule),
                },
            )),
        );

        (
            Self {
                sample_rate,
                process: Shared::clone(&process),
            },
            TimelineTrackHandle {
                save_state,
                audio_clip_indexes,
                audio_clip_params,
                process,
                sample_rate,
                coll_handle,
            },
            audio_clip_errors,
        )
    }
}

impl AudioGraphNode for TimelineTrackProcess {
    // TODO: Switch between mono and stereo based on the type of audio
    // clips on the track.
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub timeline_start: MusicalTime,
    pub timeline_duration: MusicalTime,

    pub sample_time: SampleTime,
    pub sample_duration: SampleTime,

    pub is_end: bool,
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<std::cmp::Ordering> {
        self.sample_time.partial_cmp(&other.sample_time)
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess {
    audio_clips: Shared<Vec<AudioClipProcInfo>>,
    schedule: Shared<TimelineSchedule>,
}

pub struct TimelineSchedule {
    pub schedule: Vec<(Event, TimelineEvent)>,
    pub version: u64,
}

impl TimelineSchedule {
    pub fn clone_new_version(&self) -> TimelineSchedule {
        Self {
            schedule: self.schedule.clone(),
            version: self.version + 1,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineEvent {
    AudioClipStart(usize),
    AudioClipEnd(usize),
}
