use basedrop::Owned;
use rtrb::{Consumer, Producer, RingBuffer};
use std::fmt::Debug;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use super::process_thread::EngineProcessThread;
use crate::graph::shared_pools::SharedProcessorSchedule;

/// Allocate enough for at-least 3 seconds of buffer time at the
/// highest possible sample rate.
static ALLOCATED_FRAMES_PER_CHANNEL: usize = 192_000 * 3;

/// Make sure we have a bit of time to copy the engine's output buffer to the
/// audio thread's output buffer.
static COPY_OUT_TIME_WINDOW: f64 = 0.95;

#[cfg(not(target_os = "windows"))]
pub(crate) static AUDIO_THREAD_POLL_INTERVAL: Duration = Duration::from_micros(100);
#[cfg(not(target_os = "windows"))]
// Handle worst-case scenario for thread sleep.
pub(crate) static AUDIO_THREAD_POLL_INTERVAL_BUFFERED: Duration = Duration::from_micros(140);
#[cfg(target_os = "windows")]
// The best we can do on Windows is around 1ms.
pub(crate) static AUDIO_THREAD_POLL_INTERVAL: Duration = Duration::from_micros(1200);
#[cfg(target_os = "windows")]
// Handle worst-case scenario for Windows thread sleep.
pub(crate) static AUDIO_THREAD_POLL_INTERVAL_BUFFERED: Duration = Duration::from_micros(1500);

pub struct EngineAudioThread {
    audio_to_process_channel: Owned<AudioToProcessChannelTX>,
    process_to_audio_channel: Owned<ProcessToAudioChannelRX>,

    graph_audio_in_channels: usize,
    graph_audio_out_channels: usize,

    sample_rate: u32,
    sample_rate_recip: f64,
}

impl Debug for EngineAudioThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("EngineAudioThread");

        f.field("graph_audio_in_channels", &self.graph_audio_in_channels);
        f.field("graph_audio_out_channels", &self.graph_audio_out_channels);
        f.field("sample_rate", &self.sample_rate);
        f.field("sample_rate_recip", &self.sample_rate_recip);

        f.finish()
    }
}

impl EngineAudioThread {
    pub(crate) fn new(
        schedule: SharedProcessorSchedule,
        sample_rate: u32,
        graph_audio_in_channels: usize,
        graph_audio_out_channels: usize,
        max_frames: usize,
        hard_clip_outputs: bool,
        coll_handle: &basedrop::Handle,
    ) -> (Self, EngineProcessThread) {
        assert_ne!(sample_rate, 0);

        let sample_rate_recip = 1.0 / f64::from(sample_rate);

        let (audio_to_process_tx, audio_to_process_rx) = if graph_audio_in_channels == 0 {
            let num_frames_wanted = Arc::new(AtomicUsize::new(0));

            (
                AudioToProcessChannelTX::NoInputAudio {
                    num_frames_wanted: Arc::clone(&num_frames_wanted),
                },
                AudioToProcessChannelRX::NoInputAudio { num_frames_wanted },
            )
        } else {
            let (audio_rb_tx, audio_rb_rx) =
                RingBuffer::new(graph_audio_in_channels * ALLOCATED_FRAMES_PER_CHANNEL);

            (
                AudioToProcessChannelTX::HasInputAudio { audio_rb_tx },
                AudioToProcessChannelRX::HasInputAudio { audio_rb_rx },
            )
        };

        let (process_to_audio_tx, process_to_audio_rx) = {
            let (audio_rb_tx, audio_rb_rx) =
                RingBuffer::new(graph_audio_out_channels * ALLOCATED_FRAMES_PER_CHANNEL);

            (ProcessToAudioChannelTX { audio_rb_tx }, ProcessToAudioChannelRX { audio_rb_rx })
        };

        (
            Self {
                audio_to_process_channel: Owned::new(coll_handle, audio_to_process_tx),
                process_to_audio_channel: Owned::new(coll_handle, process_to_audio_rx),
                graph_audio_in_channels,
                graph_audio_out_channels,
                sample_rate,
                sample_rate_recip,
            },
            EngineProcessThread::new(
                audio_to_process_rx,
                process_to_audio_tx,
                graph_audio_in_channels,
                graph_audio_out_channels,
                max_frames,
                hard_clip_outputs,
                schedule,
                coll_handle,
            ),
        )
    }

    pub fn process_cpal_interleaved_output_only<T: cpal::Sample>(
        &mut self,
        cpal_out_channels: usize,
        out: &mut [T],
    ) {
        let clear_output = |out: &mut [T]| {
            for s in out.iter_mut() {
                *s = T::from(&0.0);
            }
        };

        let proc_start_time = Instant::now();

        if out.len() < self.graph_audio_out_channels || cpal_out_channels == 0 {
            clear_output(out);
            return;
        }

        let total_frames = out.len() / cpal_out_channels;

        // Discard any output from previous cycles that failed to render on time.
        if !self.process_to_audio_channel.audio_rb_rx.is_empty() {
            let num_slots = self.process_to_audio_channel.audio_rb_rx.slots();

            let chunks = self.process_to_audio_channel.audio_rb_rx.read_chunk(num_slots).unwrap();
            chunks.commit_all();
        }

        match &mut *self.audio_to_process_channel {
            AudioToProcessChannelTX::HasInputAudio { audio_rb_tx } => {
                if !audio_rb_tx.is_abandoned() {
                    match audio_rb_tx.write_chunk(total_frames * self.graph_audio_in_channels) {
                        Ok(mut chunk) => {
                            let (slice_1, slice_2) = chunk.as_mut_slices();
                            slice_1.fill(0.0);
                            slice_2.fill(0.0);

                            chunk.commit_all();
                        }
                        Err(_) => {
                            log::error!(
                                "Ran out of space in audio thread to process thread audio buffer"
                            );
                            clear_output(out);
                            return;
                        }
                    }
                } else {
                    clear_output(out);
                    return;
                }
            }
            AudioToProcessChannelTX::NoInputAudio { num_frames_wanted } => {
                if Arc::strong_count(num_frames_wanted) > 1 {
                    num_frames_wanted.store(total_frames, Ordering::SeqCst);
                } else {
                    clear_output(out);
                    return;
                }
            }
        }

        let num_out_samples = total_frames * self.graph_audio_out_channels;
        if num_out_samples == 0 {
            clear_output(out);
            return;
        }

        let max_proc_time = Duration::from_secs_f64(
            total_frames as f64 * self.sample_rate_recip * COPY_OUT_TIME_WINDOW,
        );

        #[cfg(target_os = "windows")]
        let spin_sleeper = spin_sleep::SpinSleeper::default();

        loop {
            if let Ok(chunk) = self.process_to_audio_channel.audio_rb_rx.read_chunk(num_out_samples)
            {
                if cpal_out_channels == self.graph_audio_out_channels {
                    // We can simply just convert the interlaced buffer over.

                    let (slice_1, slice_2) = chunk.as_slices();

                    let out_part = &mut out[0..slice_1.len()];
                    for i in 0..slice_1.len() {
                        out_part[i] = T::from(&slice_1[i]);
                    }

                    let out_part = &mut out[slice_1.len()..slice_1.len() + slice_2.len()];
                    for i in 0..slice_2.len() {
                        out_part[i] = T::from(&slice_2[i]);
                    }
                } else {
                    let (slice_1, slice_2) = chunk.as_slices();

                    for ch_i in 0..cpal_out_channels {
                        if ch_i < self.graph_audio_out_channels {
                            for i in 0..total_frames {
                                let i2 = (i * self.graph_audio_out_channels) + ch_i;

                                let s = if i2 < slice_1.len() {
                                    slice_1[i2]
                                } else {
                                    slice_2[i2 - slice_1.len()]
                                };

                                out[(i * cpal_out_channels) + ch_i] = T::from(&s);
                            }
                        } else {
                            for i in 0..total_frames {
                                out[(i * cpal_out_channels) + ch_i] = T::from(&0.0);
                            }
                        }
                    }
                }

                chunk.commit_all();
                return;
            }

            if proc_start_time.elapsed() + AUDIO_THREAD_POLL_INTERVAL_BUFFERED >= max_proc_time {
                break;
            }

            #[cfg(not(target_os = "windows"))]
            std::thread::sleep(AUDIO_THREAD_POLL_INTERVAL);

            #[cfg(target_os = "windows")]
            spin_sleeper.sleep(AUDIO_THREAD_POLL_INTERVAL);
        }

        // The engine took too long to process.
        log::trace!("underrun");
        clear_output(out);
    }
}

enum AudioToProcessChannelTX {
    HasInputAudio { audio_rb_tx: Producer<f32> },
    NoInputAudio { num_frames_wanted: Arc<AtomicUsize> },
}

pub(super) enum AudioToProcessChannelRX {
    HasInputAudio { audio_rb_rx: Consumer<f32> },
    NoInputAudio { num_frames_wanted: Arc<AtomicUsize> },
}

pub(super) struct ProcessToAudioChannelTX {
    pub audio_rb_tx: Producer<f32>,
}

struct ProcessToAudioChannelRX {
    pub audio_rb_rx: Consumer<f32>,
}
