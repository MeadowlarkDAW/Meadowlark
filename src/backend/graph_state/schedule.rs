use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use smallvec::SmallVec;

use super::node::{MAX_AUDIO_IN_PORTS, MAX_AUDIO_OUT_PORTS};
use super::resource_pool::{MonoAudioPortBuffer, StereoAudioPortBuffer};
use super::{AudioGraphNode, MAX_BLOCKSIZE};

pub enum AudioGraphTask {
    Node {
        node: Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>,

        mono_audio_in_buffers: Vec<Shared<AtomicRefCell<MonoAudioPortBuffer>>>,
        mono_audio_out_buffers: Vec<Shared<AtomicRefCell<MonoAudioPortBuffer>>>,
        stereo_audio_in_buffers: Vec<Shared<AtomicRefCell<StereoAudioPortBuffer>>>,
        stereo_audio_out_buffers: Vec<Shared<AtomicRefCell<StereoAudioPortBuffer>>>,
    },
    // TODO: Delay compensation stuffs.
}

pub struct Schedule {
    master_out: Shared<AtomicRefCell<StereoAudioPortBuffer>>,

    tasks: Vec<AudioGraphTask>,
    proc_info: ProcInfo,
}

impl Schedule {
    pub(super) fn new(
        tasks: Vec<AudioGraphTask>,
        sample_rate: f32,
        master_out: Shared<AtomicRefCell<StereoAudioPortBuffer>>,
    ) -> Self {
        Self {
            master_out,
            tasks,
            proc_info: ProcInfo {
                frames: 0,
                sample_rate,
                sample_rate_recip: 1.0 / sample_rate,
            },
        }
    }

    /// Only to be used by the rt thread.
    pub(super) fn process(&mut self, frames: usize) {
        // TODO: Use multithreading for processing tasks.

        self.proc_info.frames = frames;

        let mut mono_audio_in_refs =
            SmallVec::<[AtomicRef<MonoAudioPortBuffer>; MAX_AUDIO_IN_PORTS]>::new();
        let mut mono_audio_out_refs =
            SmallVec::<[AtomicRefMut<MonoAudioPortBuffer>; MAX_AUDIO_OUT_PORTS]>::new();
        let mut stereo_audio_in_refs =
            SmallVec::<[AtomicRef<StereoAudioPortBuffer>; MAX_AUDIO_IN_PORTS]>::new();
        let mut stereo_audio_out_refs =
            SmallVec::<[AtomicRefMut<StereoAudioPortBuffer>; MAX_AUDIO_OUT_PORTS]>::new();

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

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = self.proc_info.frames.min(MAX_BLOCKSIZE);
        cpal_buf = &mut cpal_buf[0..frames * 2];

        for i in 0..frames {
            cpal_buf[i * 2] = cpal::Sample::from::<f32>(&src.left[i]);
            cpal_buf[(i * 2) + 1] = cpal::Sample::from::<f32>(&src.right[i]);
        }
    }
}

pub struct ProcInfo {
    /// The number of frames in every audio buffer.
    pub frames: usize,

    /// The sample rate of the stream. This remains constant for the whole lifetime of this node,
    /// so this is just provided for convenience.
    pub sample_rate: f32,

    /// The recipricol of the sample rate (1.0 / sample_rate) of the stream. This remains constant
    /// for the whole lifetime of this node, so this is just provided for convenience.
    pub sample_rate_recip: f32,
}
