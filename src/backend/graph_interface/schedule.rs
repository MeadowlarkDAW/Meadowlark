use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use rusty_daw_time::SampleRate;
use smallvec::SmallVec;

use super::node::{MAX_AUDIO_IN_PORTS, MAX_AUDIO_OUT_PORTS};
use super::resource_pool::{MonoAudioBlockBuffer, StereoAudioBlockBuffer};
use super::AudioGraphNode;
use crate::backend::timeline::TimelineTransport;

pub enum AudioGraphTask {
    Node {
        node: Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>,

        mono_audio_in_buffers: Vec<Shared<AtomicRefCell<MonoAudioBlockBuffer>>>,
        mono_audio_out_buffers: Vec<Shared<AtomicRefCell<MonoAudioBlockBuffer>>>,
        stereo_audio_in_buffers: Vec<Shared<AtomicRefCell<StereoAudioBlockBuffer>>>,
        stereo_audio_out_buffers: Vec<Shared<AtomicRefCell<StereoAudioBlockBuffer>>>,
    },
    // TODO: Delay compensation stuffs.
}

pub struct Schedule {
    master_out: Shared<AtomicRefCell<StereoAudioBlockBuffer>>,

    tasks: Vec<AudioGraphTask>,
    proc_info: ProcInfo,
}

impl Schedule {
    pub(super) fn new(
        tasks: Vec<AudioGraphTask>,
        sample_rate: SampleRate,
        master_out: Shared<AtomicRefCell<StereoAudioBlockBuffer>>,
    ) -> Self {
        Self {
            master_out,
            tasks,
            proc_info: ProcInfo::new(sample_rate),
        }
    }

    /// Only to be used by the rt thread.
    pub(super) fn process(&mut self, frames: usize, timeline_transport: &mut TimelineTransport) {
        // TODO: Use multithreading for processing tasks.

        self.proc_info.set_frames(frames);

        timeline_transport.process_declicker(&self.proc_info);

        let mut mono_audio_in_refs =
            SmallVec::<[AtomicRef<MonoAudioBlockBuffer>; MAX_AUDIO_IN_PORTS]>::new();
        let mut mono_audio_out_refs =
            SmallVec::<[AtomicRefMut<MonoAudioBlockBuffer>; MAX_AUDIO_OUT_PORTS]>::new();
        let mut stereo_audio_in_refs =
            SmallVec::<[AtomicRef<StereoAudioBlockBuffer>; MAX_AUDIO_IN_PORTS]>::new();
        let mut stereo_audio_out_refs =
            SmallVec::<[AtomicRefMut<StereoAudioBlockBuffer>; MAX_AUDIO_OUT_PORTS]>::new();

        // Where the magic happens!
        for task in self.tasks.iter() {
            match task {
                AudioGraphTask::Node {
                    node,
                    mono_audio_in_buffers,
                    mono_audio_out_buffers,
                    stereo_audio_in_buffers,
                    stereo_audio_out_buffers,
                } => {
                    // This should not panic because the rt thread is the only place these nodes
                    // are borrowed.
                    let node = &mut *AtomicRefCell::borrow_mut(node);

                    // Prepare the buffers in a safe and easy-to-use format.
                    mono_audio_in_refs.clear();
                    mono_audio_out_refs.clear();
                    stereo_audio_in_refs.clear();
                    stereo_audio_out_refs.clear();
                    for b in mono_audio_in_buffers.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        mono_audio_in_refs.push(AtomicRefCell::borrow(b));
                    }
                    for b in mono_audio_out_buffers.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        mono_audio_out_refs.push(AtomicRefCell::borrow_mut(b));
                    }
                    for b in stereo_audio_in_buffers.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        stereo_audio_in_refs.push(AtomicRefCell::borrow(b));
                    }
                    for b in stereo_audio_out_buffers.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        stereo_audio_out_refs.push(AtomicRefCell::borrow_mut(b));
                    }

                    node.process(
                        &self.proc_info,
                        timeline_transport,
                        mono_audio_in_refs.as_slice(),
                        mono_audio_out_refs.as_mut_slice(),
                        stereo_audio_in_refs.as_slice(),
                        stereo_audio_out_refs.as_mut_slice(),
                    );
                }
            }
        }
    }

    pub(super) fn copy_master_output_to_cpal<T: cpal::Sample>(&self, mut cpal_buf: &mut [T]) {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        let src = &mut *AtomicRefCell::borrow_mut(&self.master_out);
        cpal_buf = &mut cpal_buf[0..self.proc_info.frames() * 2];

        for i in 0..self.proc_info.frames() {
            cpal_buf[i * 2] = cpal::Sample::from::<f32>(&src.left[i]);
            cpal_buf[(i * 2) + 1] = cpal::Sample::from::<f32>(&src.right[i]);
        }
    }
}

pub use proc_info::ProcInfo;

/// This is separated into a module to hopefully allow the compiler to reason that `frames`
/// will always be less than or equal to `MAX_BLOCKSIZE`, allowing for bounds checking to be
/// elided on loops which use the `frames()` method.
mod proc_info {
    use crate::backend::MAX_BLOCKSIZE;
    use rusty_daw_time::SampleRate;

    #[derive(Debug, Clone, Copy)]
    pub struct ProcInfo {
        /// The sample rate of the stream. This remains constant for the whole lifetime of this node,
        /// so this is just provided for convenience.
        pub sample_rate: SampleRate,

        /// The recipricol of the sample rate (1.0 / sample_rate) of the stream. This remains constant
        /// for the whole lifetime of this node, so this is just provided for convenience.
        pub sample_rate_recip: f64,

        frames: usize,
    }

    impl ProcInfo {
        pub(super) fn new(sample_rate: SampleRate) -> Self {
            Self {
                sample_rate,
                sample_rate_recip: sample_rate.recip(),
                frames: 0,
            }
        }

        /// This is separated into a function to hopefully allow the compiler to reason that this
        /// will always be less than or equal to `MAX_BLOCKSIZE`, allowing for bounds checking to be
        /// elided on loops which use the `frames()` method.
        pub(super) fn set_frames(&mut self, frames: usize) {
            self.frames = frames.min(MAX_BLOCKSIZE);
        }

        /// The number of audio frames in this current process block.
        ///
        /// This will always be less than or equal to `MAX_BLOCKSIZE`.
        ///
        /// This is separated into a function to hopefully allow the compiler to reason that this
        /// will always be less than or equal to `MAX_BLOCKSIZE`, allowing for bounds checking to be
        /// elided on loops which use this.
        #[inline]
        pub fn frames(&self) -> usize {
            self.frames
        }
    }
}
