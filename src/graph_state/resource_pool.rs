use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared};

use super::node::AudioGraphNode;

pub struct ResourcePool {
    // Using AtomicRefCell because these resources are only ever borrowed by
    // the rt thread. We keep these pointers in a non-rt thread so we can
    // reconstruct a new schedule to send to the rt thread whenever the
    // graph is recompiled.
    pub(super) nodes: Vec<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>,
    pub(super) mono_audio_buffers: Vec<Shared<AtomicRefCell<MonoAudioPortBuffer>>>,
    pub(super) stereo_audio_buffers: Vec<Shared<AtomicRefCell<StereoAudioPortBuffer>>>,

    max_audio_frames: usize,
    coll_handle: Handle,
}

impl Clone for ResourcePool {
    fn clone(&self) -> Self {
        let mut nodes =
            Vec::<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>::with_capacity(self.nodes.len());
        let mut mono_audio_buffers =
            Vec::<Shared<AtomicRefCell<MonoAudioPortBuffer>>>::with_capacity(
                self.mono_audio_buffers.len(),
            );
        let mut stereo_audio_buffers =
            Vec::<Shared<AtomicRefCell<StereoAudioPortBuffer>>>::with_capacity(
                self.stereo_audio_buffers.len(),
            );

        for node in self.nodes.iter() {
            nodes.push(Shared::clone(node));
        }
        for audio_buffer in self.mono_audio_buffers.iter() {
            mono_audio_buffers.push(Shared::clone(audio_buffer));
        }
        for audio_buffer in self.stereo_audio_buffers.iter() {
            stereo_audio_buffers.push(Shared::clone(audio_buffer));
        }

        Self {
            nodes,
            mono_audio_buffers,
            stereo_audio_buffers,
            max_audio_frames: self.max_audio_frames,
            coll_handle: self.coll_handle.clone(),
        }
    }
}

impl ResourcePool {
    /// Create a new resource pool. Only to be used by the non-rt thread.
    pub(super) fn new(max_audio_frames: usize, coll_handle: Handle) -> Self {
        Self {
            nodes: Vec::new(),
            mono_audio_buffers: Vec::new(),
            stereo_audio_buffers: Vec::new(),
            max_audio_frames,
            coll_handle: coll_handle,
        }
    }

    /// Add a new audio graph nodes to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_node(&mut self, new_node: Box<dyn AudioGraphNode>) {
        self.nodes
            .push(Shared::new(&self.coll_handle, AtomicRefCell::new(new_node)));
    }

    /// Remove nodes from the pool. Only to be used by the non-rt thread.
    pub(super) fn remove_node(&mut self, node_index: usize) -> Result<(), ()> {
        if node_index < self.nodes.len() {
            self.nodes.remove(node_index);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Add new mono audio port buffer to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_mono_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.mono_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(MonoAudioPortBuffer::new(self.max_audio_frames)),
            ));
        }
    }

    /// Add new stereo audio port buffer to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_stereo_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.stereo_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(StereoAudioPortBuffer::new(self.max_audio_frames)),
            ));
        }
    }

    /// Remove audio buffers from the pool. Only to be used by the non-rt thread.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_mono_audio_buffers(&mut self, n_to_remove: usize) -> Result<(), ()> {
        if n_to_remove <= self.mono_audio_buffers.len() {
            for _ in 0..n_to_remove {
                let _ = self.mono_audio_buffers.pop();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Remove audio buffers from the pool. Only to be used by the non-rt thread.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_stereo_audio_buffers(&mut self, n_to_remove: usize) -> Result<(), ()> {
        if n_to_remove <= self.stereo_audio_buffers.len() {
            for _ in 0..n_to_remove {
                let _ = self.stereo_audio_buffers.pop();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Only to be used by the rt thread.
    pub fn clear_and_resize_all_buffers(&mut self, frames: usize) {
        if frames > self.max_audio_frames {
            log::warn!(
                "Rt thread resizing audio buffers to {} frames when maximum is {} frames.",
                frames,
                self.max_audio_frames
            );
        }

        for b in self.mono_audio_buffers.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear_and_resize(frames);
        }
        for b in self.stereo_audio_buffers.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear_and_resize(frames);
        }
    }
}

#[derive(Debug)]
pub struct MonoAudioPortBuffer {
    buffer: Vec<f32>,
}

impl MonoAudioPortBuffer {
    fn new(max_frames: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_frames),
        }
    }

    fn clear_and_resize(&mut self, frames: usize) {
        self.buffer.clear();
        self.buffer.resize(frames, 0.0);
    }

    pub fn get(&self) -> &[f32] {
        &self.buffer
    }

    pub fn get_mut(&mut self) -> &mut [f32] {
        &mut self.buffer
    }

    pub fn copy_from(&mut self, src: &MonoAudioPortBuffer) {
        self.buffer.copy_from_slice(&src.buffer);
    }

    pub fn copy_from_stereo_left(&mut self, src: &StereoAudioPortBuffer) {
        self.buffer.copy_from_slice(&src.buffer_l);
    }
    pub fn copy_from_stereo_right(&mut self, src: &StereoAudioPortBuffer) {
        self.buffer.copy_from_slice(&src.buffer_r);
    }
}

#[derive(Debug)]
pub struct StereoAudioPortBuffer {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
}

impl StereoAudioPortBuffer {
    fn new(max_frames: usize) -> Self {
        Self {
            buffer_l: Vec::with_capacity(max_frames),
            buffer_r: Vec::with_capacity(max_frames),
        }
    }

    fn clear_and_resize(&mut self, frames: usize) {
        self.buffer_l.clear();
        self.buffer_l.resize(frames, 0.0);

        self.buffer_r.clear();
        self.buffer_r.resize(frames, 0.0);
    }

    pub fn left(&self) -> &[f32] {
        &self.buffer_l
    }
    pub fn right(&self) -> &[f32] {
        &self.buffer_r
    }

    pub fn left_mut(&mut self) -> &mut [f32] {
        &mut self.buffer_l
    }
    pub fn right_mut(&mut self) -> &[f32] {
        &mut self.buffer_r
    }

    pub fn left_right(&self) -> (&[f32], &[f32]) {
        (&self.buffer_l, &self.buffer_r)
    }
    pub fn left_right_mut(&mut self) -> (&mut [f32], &mut [f32]) {
        (&mut self.buffer_l, &mut self.buffer_r)
    }

    pub fn copy_from(&mut self, src: &StereoAudioPortBuffer) {
        self.buffer_l.copy_from_slice(&src.buffer_l);
        self.buffer_r.copy_from_slice(&src.buffer_r);
    }

    pub fn copy_from_mono_to_left(&mut self, src: &MonoAudioPortBuffer) {
        self.buffer_l.copy_from_slice(&src.buffer);
    }
    pub fn copy_from_mono_to_right(&mut self, src: &MonoAudioPortBuffer) {
        self.buffer_r.copy_from_slice(&src.buffer);
    }
}
