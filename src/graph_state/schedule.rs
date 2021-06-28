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
    CopyMonoAudioBuffer {
        src: Shared<AtomicRefCell<MonoAudioPortBuffer>>,
        dst: Shared<AtomicRefCell<MonoAudioPortBuffer>>,
    },
    CopyStereoAudioBuffer {
        src: Shared<AtomicRefCell<StereoAudioPortBuffer>>,
        dst: Shared<AtomicRefCell<StereoAudioPortBuffer>>,
    },
    // Mix the source audio buffers into the destination audio buffer.
    MixMonoAudioBuffers {
        src: Vec<Shared<AtomicRefCell<MonoAudioPortBuffer>>>,
        dst: Shared<AtomicRefCell<MonoAudioPortBuffer>>,
    },
    // Mix the source audio buffers into the destination audio buffer.
    MixStereoAudioBuffers {
        src: Vec<Shared<AtomicRefCell<StereoAudioPortBuffer>>>,
        dst: Shared<AtomicRefCell<StereoAudioPortBuffer>>,
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
                AudioGraphTask::CopyMonoAudioBuffer { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are borrowed.
                    let dst = &mut *AtomicRefCell::borrow_mut(dst);
                    let src = &*AtomicRefCell::borrow(src);
                    dst.copy_from(src);
                }
                AudioGraphTask::CopyStereoAudioBuffer { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are borrowed.
                    let dst = &mut *AtomicRefCell::borrow_mut(dst);
                    let src = &*AtomicRefCell::borrow(src);
                    dst.copy_from(src);
                }
                AudioGraphTask::MixMonoAudioBuffers { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are borrowed.
                    let dst_ref = &mut *AtomicRefCell::borrow_mut(dst);
                    let dst_ch = dst_ref.get_mut();
                    for src_b in src.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        let src_ref = &*AtomicRefCell::borrow(src_b);
                        let src_ch = src_ref.get();

                        // TODO: Manual SIMD to take advantage of AVX.

                        for i in 0..self.proc_info.frames {
                            // Safe because the scheduler ensures that all buffers have the length `proc_info.frames`.
                            unsafe {
                                *dst_ch.get_unchecked_mut(i) += *src_ch.get_unchecked(i);
                            }
                        }
                    }
                }
                AudioGraphTask::MixStereoAudioBuffers { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are borrowed.
                    let dst_ref = &mut *AtomicRefCell::borrow_mut(dst);
                    let (dst_l, dst_r) = dst_ref.left_right_mut();
                    for src_b in src.iter() {
                        // This should not panic because the rt thread is the only place these buffers
                        // are borrowed.
                        let src_ref = &*AtomicRefCell::borrow(src_b);
                        let (src_l, src_r) = src_ref.left_right();

                        // TODO: Manual SIMD to take advantage of AVX.

                        for i in 0..self.proc_info.frames {
                            // Safe because the scheduler ensures that all buffers have the length `proc_info.frames`.
                            unsafe {
                                *dst_l.get_unchecked_mut(i) += *src_l.get_unchecked(i);
                                *dst_r.get_unchecked_mut(i) += *src_r.get_unchecked(i);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(super) fn copy_master_output_to_cpal<T: cpal::Sample>(&self, cpal_buf: &mut [T]) {
        assert_eq!(cpal_buf.len(), self.proc_info.frames * 2);

        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        let src = &mut *AtomicRefCell::borrow_mut(&self.master_out);
        let (src_l, src_r) = src.left_right_mut();

        for i in 0..self.proc_info.frames {
            // Safe because the scheduler ensures that all buffers have the length `self.proc_info.frames`, and
            // we asserted that the cpal buffer has the correct amount of frames.
            unsafe {
                *cpal_buf.get_unchecked_mut(i * 2) =
                    cpal::Sample::from::<f32>(src_l.get_unchecked(i));
                *cpal_buf.get_unchecked_mut((i * 2) + 1) =
                    cpal::Sample::from::<f32>(src_r.get_unchecked(i));
            }
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
