use atomic_refcell::AtomicRefCell;
use basedrop::Shared;
use rusty_daw_time::SampleRate;

use super::{AudioGraphTask, StereoBlockBuffer};
use crate::backend::timeline::TimelineTransport;
use crate::backend::MAX_BLOCKSIZE;

pub struct Schedule {
    master_out: Shared<AtomicRefCell<StereoBlockBuffer<f32>>>,

    tasks: Vec<AudioGraphTask>,
    proc_info: ProcInfo,
}

impl Schedule {
    pub(super) fn new(
        tasks: Vec<AudioGraphTask>,
        sample_rate: SampleRate,
        master_out: Shared<AtomicRefCell<StereoBlockBuffer<f32>>>,
    ) -> Self {
        Self { master_out, tasks, proc_info: ProcInfo::new(sample_rate) }
    }

    /// Only to be used by the rt thread.
    pub(super) fn process(&mut self, frames: usize, timeline_transport: &mut TimelineTransport) {
        // TODO: Use multithreading for processing tasks.

        self.proc_info.set_frames(frames);

        timeline_transport.process_declicker(&self.proc_info);

        // Where the magic happens!
        for task in self.tasks.iter_mut() {
            match task {
                AudioGraphTask::Node { node, proc_buffers } => {
                    // This should not panic because the rt thread is the only place these nodes
                    // are borrowed.
                    //
                    // TODO: Use unsafe instead of runtime checking? It would be more efficient,
                    // but in theory a bug in the scheduler could try and assign the same node
                    // twice in parallel tasks, so it would be nice to detect if that happens.
                    let node = &mut *AtomicRefCell::borrow_mut(node);

                    node.process(&self.proc_info, timeline_transport, proc_buffers);
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
        Self { sample_rate, sample_rate_recip: sample_rate.recip(), frames: 0 }
    }

    #[inline]
    pub(super) fn set_frames(&mut self, frames: usize) {
        self.frames = frames.min(MAX_BLOCKSIZE);
    }

    /// The number of audio frames in this current process block.
    ///
    /// This will always be less than or equal to `MAX_BLOCKSIZE`.
    ///
    /// Note, for optimization purposes, this internally looks like
    /// `self.frames.min(MAX_BLOCKSIZE)`. This allows the compiler to
    /// safely optimize loops over buffers with length `MAX_BLOCKSIZE`
    /// by eliding all bounds checking and allowing for more aggressive
    /// auto-vectorization optimizations. If you need to use this multiple
    /// times within the same function, please only call this once and store
    /// it in a local variable to avoid running this internal check every
    /// subsequent time.
    #[inline]
    pub fn frames(&self) -> usize {
        self.frames.min(MAX_BLOCKSIZE)
    }
}
