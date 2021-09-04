use atomic_refcell::AtomicRefCell;
use audio_graph::NodeRef;
use basedrop::{Handle, Shared};

use crate::backend::MAX_BLOCKSIZE;

use super::{node::AudioGraphNode, MonoBlockBuffer, StereoBlockBuffer};

#[derive(Clone)]
pub struct GraphResourcePool {
    // Using AtomicRefCell because these resources are only ever borrowed by
    // the rt thread. We keep these pointers in a non-rt thread so we can
    // cheaply clone and reconstruct a new schedule to send to the rt thread whenever the
    // graph is recompiled (only need to copy pointers instead of whole Vecs).
    pub(super) nodes: Vec<Option<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>>,
    pub(super) mono_block_buffers_f32: Vec<Shared<AtomicRefCell<MonoBlockBuffer<f32>>>>,
    pub(super) stereo_block_buffers_f32: Vec<Shared<AtomicRefCell<StereoBlockBuffer<f32>>>>,

    coll_handle: Handle,
}

impl GraphResourcePool {
    /// Create a new resource pool. Only to be used by the non-rt thread.
    pub(super) fn new(coll_handle: Handle) -> Self {
        Self {
            nodes: Vec::new(),
            mono_block_buffers_f32: Vec::new(),
            stereo_block_buffers_f32: Vec::new(),
            coll_handle,
        }
    }

    pub(super) fn add_node(&mut self, node_ref: NodeRef, new_node: Box<dyn AudioGraphNode>) {
        let index: usize = node_ref.into();
        while index >= self.nodes.len() {
            self.nodes.push(None);
        }

        self.nodes[index] = Some(Shared::new(&self.coll_handle, AtomicRefCell::new(new_node)));
    }

    pub(super) fn remove_node(&mut self, node_ref: NodeRef) {
        let index: usize = node_ref.into();
        self.nodes[index] = None;
    }

    pub(super) fn add_mono_audio_block_buffers_f32(&mut self, n_new_block_buffers: usize) {
        for _ in 0..n_new_block_buffers {
            self.mono_block_buffers_f32
                .push(Shared::new(&self.coll_handle, AtomicRefCell::new(MonoBlockBuffer::new())));
        }
    }

    pub(super) fn add_stereo_audio_block_buffers_f32(&mut self, n_new_block_buffers: usize) {
        for _ in 0..n_new_block_buffers {
            self.stereo_block_buffers_f32
                .push(Shared::new(&self.coll_handle, AtomicRefCell::new(StereoBlockBuffer::new())));
        }
    }

    /// Remove audio buffers from the pool.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_mono_block_buffers_f32(&mut self, n_to_remove: usize) -> Result<(), ()> {
        if n_to_remove <= self.mono_block_buffers_f32.len() {
            for _ in 0..n_to_remove {
                let _ = self.mono_block_buffers_f32.pop();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Remove audio buffers from the pool.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_stereo_block_buffers_f32(&mut self, n_to_remove: usize) -> Result<(), ()> {
        if n_to_remove <= self.stereo_block_buffers_f32.len() {
            for _ in 0..n_to_remove {
                let _ = self.stereo_block_buffers_f32.pop();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Only to be used by the rt thread.
    pub fn clear_all_buffers(&mut self, frames: usize) {
        let frames = frames.min(MAX_BLOCKSIZE);

        for b in self.mono_block_buffers_f32.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear_frames(frames);
        }
        for b in self.stereo_block_buffers_f32.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear_frames(frames);
        }
    }
}
