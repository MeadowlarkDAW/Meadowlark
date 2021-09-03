use atomic_refcell::AtomicRefCell;
use audio_graph::NodeRef;
use basedrop::{Handle, Shared};

use super::{node::AudioGraphNode, MonoAudioBlockBuffer, StereoAudioBlockBuffer};

#[derive(Clone)]
pub struct GraphResourcePool {
    // Using AtomicRefCell because these resources are only ever borrowed by
    // the rt thread. We keep these pointers in a non-rt thread so we can
    // cheaply clone and reconstruct a new schedule to send to the rt thread whenever the
    // graph is recompiled (only need to copy pointers instead of whole Vecs).
    pub(super) nodes: Vec<Option<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>>,
    pub(super) mono_audio_buffers: Vec<Shared<AtomicRefCell<MonoAudioBlockBuffer>>>,
    pub(super) stereo_audio_buffers: Vec<Shared<AtomicRefCell<StereoAudioBlockBuffer>>>,

    coll_handle: Handle,
}

impl GraphResourcePool {
    /// Create a new resource pool. Only to be used by the non-rt thread.
    pub(super) fn new(coll_handle: Handle) -> Self {
        Self {
            nodes: Vec::new(),
            mono_audio_buffers: Vec::new(),
            stereo_audio_buffers: Vec::new(),
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

    pub(super) fn add_mono_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.mono_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(MonoAudioBlockBuffer::new()),
            ));
        }
    }

    pub(super) fn add_stereo_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.stereo_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(StereoAudioBlockBuffer::new()),
            ));
        }
    }

    /// Remove audio buffers from the pool.
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

    /// Remove audio buffers from the pool.
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
    pub fn clear_all_buffers(&mut self) {
        for b in self.mono_audio_buffers.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear();
        }
        for b in self.stereo_audio_buffers.iter() {
            // Should not panic because the rt thread is the only thread that ever borrows resources.
            let b = &mut *AtomicRefCell::borrow_mut(b);

            b.clear();
        }
    }
}
