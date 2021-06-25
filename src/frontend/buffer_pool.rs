use basedrop::{Handle, Shared};
use std::ops::Range;

pub struct BufferPool {
    pub audio_buffers: Vec<Shared<Vec<f32>>>,
}

impl BufferPool {
    pub(super) fn new() -> Self {
        Self {
            audio_buffers: Vec::new(),
        }
    }

    /// Add new audio buffers to the pool.
    pub(super) fn add_new_audio_buffers(
        &mut self,
        n_new_buffers: usize,
        max_frames: usize,
        coll_handle: &Handle,
    ) {
        // Create a new Vec of pointers to the buffers.
        let mut new_vec =
            Vec::<Shared<Vec<f32>>>::with_capacity(self.audio_buffers.len() + n_new_buffers);

        // Clone the old Vec.
        for buffer in self.audio_buffers.iter() {
            new_vec.push(Shared::clone(buffer));
        }

        // Add the new buffers.
        for _ in 0..n_new_buffers {
            new_vec.push(Shared::new(coll_handle, Vec::with_capacity(max_frames)));
        }

        // Our reference to the old Vec of pointers is dropped here and is replaced with the new one.
        // The old Vec will be fully dropped once the rt thread finishes using it.
        self.audio_buffers = new_vec;
    }

    /// Remove audio buffers from the pool.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the buffers to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_audio_buffers(&mut self, range: Range<usize>) -> Result<(), ()> {
        if range.is_empty() || range.end > self.audio_buffers.len() {
            return Err(());
        }

        // Create a new Vec of pointers to the buffers.
        let mut new_vec = Vec::<Shared<Vec<f32>>>::with_capacity(
            self.audio_buffers.len() - (range.end - range.start),
        );

        // Clone only the elements not in the range.
        for (i, buffer) in self.audio_buffers.iter().enumerate() {
            if !range.contains(&i) {
                new_vec.push(Shared::clone(buffer));
            }
        }

        // Our reference to the old Vec of pointers is dropped here and is replaced with the new one.
        // The old Vec will be fully dropped once the rt thread finishes using it.
        self.audio_buffers = new_vec;

        Ok(())
    }
}
