use basedrop::Owned;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::graph::shared_pools::SharedProcessorSchedule;

use super::audio_thread::{
    AudioToProcessChannelRX, ProcessToAudioChannelTX, AUDIO_THREAD_POLL_INTERVAL,
};

pub(crate) struct EngineProcessThread {
    audio_to_process_channel: Owned<AudioToProcessChannelRX>,
    process_to_audio_channel: Owned<ProcessToAudioChannelTX>,

    graph_audio_in_channels: usize,
    graph_audio_out_channels: usize,

    audio_in_temp_buffer: Owned<Vec<f32>>,
    audio_out_temp_buffer: Owned<Vec<f32>>,

    hard_clip_outputs: bool,

    schedule: SharedProcessorSchedule,
}

impl EngineProcessThread {
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub(super) fn new(
        audio_to_process_channel: AudioToProcessChannelRX,
        process_to_audio_channel: ProcessToAudioChannelTX,
        graph_audio_in_channels: usize,
        graph_audio_out_channels: usize,
        max_frames: usize,
        hard_clip_outputs: bool,
        schedule: SharedProcessorSchedule,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        Self {
            audio_to_process_channel: Owned::new(coll_handle, audio_to_process_channel),
            process_to_audio_channel: Owned::new(coll_handle, process_to_audio_channel),
            graph_audio_in_channels,
            graph_audio_out_channels,
            audio_in_temp_buffer: Owned::new(
                coll_handle,
                Vec::with_capacity(graph_audio_in_channels * max_frames),
            ),
            audio_out_temp_buffer: Owned::new(
                coll_handle,
                Vec::with_capacity(graph_audio_out_channels * max_frames),
            ),
            hard_clip_outputs,
            schedule,
        }
    }

    pub fn run(&mut self, run: Arc<AtomicBool>) {
        #[cfg(target_os = "windows")]
        let spin_sleeper = spin_sleep::SpinSleeper::default();

        while run.load(Ordering::Relaxed) {
            let num_frames = match &mut *self.audio_to_process_channel {
                AudioToProcessChannelRX::HasInputAudio { audio_rb_rx } => {
                    if !audio_rb_rx.is_abandoned() {
                        let num_samples = audio_rb_rx.slots();

                        if num_samples == 0 {
                            #[cfg(not(target_os = "windows"))]
                            std::thread::sleep(AUDIO_THREAD_POLL_INTERVAL);

                            #[cfg(target_os = "windows")]
                            spin_sleeper.sleep(AUDIO_THREAD_POLL_INTERVAL);

                            continue;
                        }

                        let chunk = audio_rb_rx.read_chunk(num_samples).unwrap();

                        let (slice_1, slice_2) = chunk.as_slices();

                        self.audio_in_temp_buffer.clear();
                        self.audio_in_temp_buffer.extend_from_slice(slice_1);
                        self.audio_in_temp_buffer.extend_from_slice(slice_2);

                        chunk.commit_all();

                        num_samples / self.graph_audio_in_channels
                    } else {
                        run.store(false, Ordering::Relaxed);
                        break;
                    }
                }
                AudioToProcessChannelRX::NoInputAudio { num_frames_wanted } => {
                    if Arc::strong_count(num_frames_wanted) > 1 {
                        let num_frames = num_frames_wanted.swap(0, Ordering::SeqCst);

                        if num_frames == 0 {
                            #[cfg(not(target_os = "windows"))]
                            std::thread::sleep(AUDIO_THREAD_POLL_INTERVAL);

                            #[cfg(target_os = "windows")]
                            spin_sleeper.sleep(AUDIO_THREAD_POLL_INTERVAL);

                            continue;
                        }

                        num_frames
                    } else {
                        run.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            };

            self.audio_out_temp_buffer.clear();
            self.audio_out_temp_buffer.resize(num_frames * self.graph_audio_out_channels, 0.0);

            self.schedule
                .process_interleaved(&self.audio_in_temp_buffer, &mut self.audio_out_temp_buffer);

            if self.hard_clip_outputs {
                for smp in self.audio_out_temp_buffer.iter_mut() {
                    *smp = smp.clamp(-1.0, 1.0);
                }
            }

            match self
                .process_to_audio_channel
                .audio_rb_tx
                .write_chunk(num_frames * self.graph_audio_out_channels)
            {
                Ok(mut chunk) => {
                    let (slice_1, slice_2) = chunk.as_mut_slices();

                    let out_part = &self.audio_out_temp_buffer[0..slice_1.len()];
                    slice_1.copy_from_slice(&out_part[..slice_1.len()]);

                    let out_part =
                        &self.audio_out_temp_buffer[slice_1.len()..slice_1.len() + slice_2.len()];
                    slice_2.copy_from_slice(&out_part[..slice_2.len()]);

                    chunk.commit_all();
                }
                Err(_) => {
                    log::error!("Ran out of space in process thread to audio thread audio buffer");
                    return;
                }
            }
        }

        // Make sure we drop all plugin processors in the process thread
        // when deactivating the engine.
        self.schedule.deactivate();
    }
}
