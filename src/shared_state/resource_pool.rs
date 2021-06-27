use basedrop::{Handle, Shared};

use super::node::AudioGraphNode;

pub struct ResourcePool {
    pub nodes: Vec<Shared<Box<dyn AudioGraphNode>>>,
    pub audio_buffers: Vec<Shared<Vec<f32>>>,

    max_audio_frames: usize,
}

impl Clone for ResourcePool {
    fn clone(&self) -> Self {
        let mut nodes = Vec::<Shared<Box<dyn AudioGraphNode>>>::with_capacity(self.nodes.len());
        let mut audio_buffers = Vec::<Shared<Vec<f32>>>::with_capacity(self.audio_buffers.len());

        for node in self.nodes.iter() {
            nodes.push(Shared::clone(node));
        }
        for audio_buffer in self.audio_buffers.iter() {
            audio_buffers.push(Shared::clone(audio_buffer));
        }

        Self {
            nodes,
            audio_buffers,
            max_audio_frames: self.max_audio_frames,
        }
    }
}

impl ResourcePool {
    /// Create a new resource pool. Only to be used by the non-rt thread.
    pub(super) fn new(max_audio_frames: usize) -> Self {
        Self {
            nodes: Vec::new(),
            audio_buffers: Vec::new(),
            max_audio_frames,
        }
    }

    /// Add a new audio graph nodes to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_node(&mut self, new_node: Shared<Box<dyn AudioGraphNode>>) {
        self.nodes.push(new_node);
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

    /// Add new audio buffers to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_new_audio_buffers(&mut self, n_new_buffers: usize, coll_handle: &Handle) {
        for _ in 0..n_new_buffers {
            self.audio_buffers.push(Shared::new(
                coll_handle,
                Vec::with_capacity(self.max_audio_frames),
            ));
        }
    }

    /// Remove audio buffers from the pool. Only to be used by the non-rt thread.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_audio_buffers(&mut self, n_to_remove: usize) -> Result<(), ()> {
        if n_to_remove <= self.audio_buffers.len() {
            for _ in 0..n_to_remove {
                let _ = self.audio_buffers.pop();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Only to be used by the rt thread.
    pub fn clear_all_buffers(&mut self, frames: usize) {
        if frames > self.max_audio_frames {
            log::warn!(
                "Rt thread resizing audio buffers to {} frames when maximum is {} frames.",
                frames,
                self.max_audio_frames
            );
        }

        for b in self.audio_buffers.iter_mut() {
            // This should not panic because the rt thread is the only one that mutate these buffers.
            let b = Shared::get_mut(b).unwrap();

            b.clear();
            b.resize(frames, 0.0);
        }
    }
}
