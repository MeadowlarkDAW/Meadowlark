use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared};

use super::{node::AudioGraphNode, MAX_BLOCKSIZE};

pub struct ResourcePool {
    // Using AtomicRefCell because these resources are only ever borrowed by
    // the rt thread. We keep these pointers in a non-rt thread so we can
    // reconstruct a new schedule to send to the rt thread whenever the
    // graph is recompiled.
    pub(super) nodes: Vec<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>,
    pub(super) mono_audio_buffers: Vec<Shared<AtomicRefCell<MonoAudioPortBuffer>>>,
    pub(super) stereo_audio_buffers: Vec<Shared<AtomicRefCell<StereoAudioPortBuffer>>>,

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
            coll_handle: self.coll_handle.clone(),
        }
    }
}

impl ResourcePool {
    /// Create a new resource pool. Only to be used by the non-rt thread.
    pub(super) fn new(coll_handle: Handle) -> Self {
        Self {
            nodes: Vec::new(),
            mono_audio_buffers: Vec::new(),
            stereo_audio_buffers: Vec::new(),
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
                AtomicRefCell::new(MonoAudioPortBuffer::new()),
            ));
        }
    }

    /// Add new stereo audio port buffer to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_stereo_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.stereo_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(StereoAudioPortBuffer::new()),
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
        assert!(frames <= MAX_BLOCKSIZE);

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
    buffer: [f32; MAX_BLOCKSIZE],
    len: usize,
}

impl MonoAudioPortBuffer {
    fn new() -> Self {
        Self {
            buffer: [0.0; MAX_BLOCKSIZE],
            len: 0,
        }
    }

    fn clear_and_resize(&mut self, frames: usize) {
        self.len = frames.min(MAX_BLOCKSIZE);
        for i in 0..self.len {
            self.buffer[i] = 0.0;
        }
    }

    pub fn get(&self) -> &[f32] {
        &self.buffer[0..self.len]
    }

    pub fn get_mut(&mut self) -> &mut [f32] {
        &mut self.buffer[0..self.len]
    }

    pub fn copy_from(&mut self, src: &MonoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer[0..len].copy_from_slice(&src.buffer[0..len]);
    }

    pub fn copy_from_stereo_left(&mut self, src: &StereoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer[0..len].copy_from_slice(&src.buffer_l[0..len]);
    }
    pub fn copy_from_stereo_right(&mut self, src: &StereoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer[0..len].copy_from_slice(&src.buffer_r[0..len]);
    }
}

#[derive(Debug)]
pub struct StereoAudioPortBuffer {
    buffer_l: [f32; MAX_BLOCKSIZE],
    buffer_r: [f32; MAX_BLOCKSIZE],
    len: usize,
}

impl StereoAudioPortBuffer {
    fn new() -> Self {
        Self {
            buffer_l: [0.0; MAX_BLOCKSIZE],
            buffer_r: [0.0; MAX_BLOCKSIZE],
            len: 0,
        }
    }

    fn clear_and_resize(&mut self, frames: usize) {
        self.len = frames.min(MAX_BLOCKSIZE);
        for i in 0..self.len {
            self.buffer_l[i] = 0.0;
        }
        for i in 0..self.len {
            self.buffer_r[i] = 0.0;
        }
    }

    pub fn left(&self) -> &[f32] {
        &self.buffer_l[0..self.len]
    }
    pub fn right(&self) -> &[f32] {
        &self.buffer_r[0..self.len]
    }

    pub fn left_mut(&mut self) -> &mut [f32] {
        &mut self.buffer_l[0..self.len]
    }
    pub fn right_mut(&mut self) -> &[f32] {
        &mut self.buffer_r[0..self.len]
    }

    pub fn left_right(&self) -> (&[f32], &[f32]) {
        (&self.buffer_l[0..self.len], &self.buffer_r[0..self.len])
    }
    pub fn left_right_mut(&mut self) -> (&mut [f32], &mut [f32]) {
        (
            &mut self.buffer_l[0..self.len],
            &mut self.buffer_r[0..self.len],
        )
    }

    pub fn copy_from(&mut self, src: &StereoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer_l[0..len].copy_from_slice(&src.buffer_l[0..len]);
        &mut self.buffer_r[0..len].copy_from_slice(&src.buffer_r[0..len]);
    }

    pub fn copy_from_mono_to_left(&mut self, src: &MonoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer_l[0..len].copy_from_slice(&src.buffer[0..len]);
    }
    pub fn copy_from_mono_to_right(&mut self, src: &MonoAudioPortBuffer) {
        let len = self.len.min(src.len);
        &mut self.buffer_r[0..len].copy_from_slice(&src.buffer[0..len]);
    }
}
