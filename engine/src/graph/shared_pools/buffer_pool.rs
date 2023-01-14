use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::{DebugBufferID, DebugBufferType, SharedBuffer};

use crate::plugin_host::event_io_buffers::NoteIoEvent;

pub(crate) struct BufferPool<T: Clone + Copy + Send + Sync + 'static> {
    pool: Vec<SharedBuffer<T>>,
    buffer_size: usize,
    buffer_type: DebugBufferType,
    collection_handle: basedrop::Handle,
}

impl<T: Clone + Copy + Send + Sync + 'static> BufferPool<T> {
    fn new(
        buffer_size: usize,
        buffer_type: DebugBufferType,
        collection_handle: basedrop::Handle,
    ) -> Self {
        assert_ne!(buffer_size, 0);

        Self { pool: Vec::new(), buffer_size, collection_handle, buffer_type }
    }

    #[inline]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn buffer_at_index(&mut self, index: usize) -> SharedBuffer<T> {
        if index >= self.pool.len() {
            let mut current_generated_index = self.pool.len() as u32;

            self.pool.resize_with(index + 1, || {
                let buf = SharedBuffer::with_capacity(
                    self.buffer_size,
                    DebugBufferID { index: current_generated_index, buffer_type: self.buffer_type },
                    &self.collection_handle,
                );
                current_generated_index += 1;
                buf
            })
        }

        self.pool[index].clone()
    }
}

impl<T: Clone + Copy + Send + Sync + 'static + Default> BufferPool<T> {
    pub fn initialized_buffer_at_index(&mut self, index: usize) -> SharedBuffer<T> {
        if index >= self.pool.len() {
            let mut current_generated_index = self.pool.len() as u32;

            self.pool.resize_with(index + 1, || {
                let buf = SharedBuffer::new(
                    self.buffer_size,
                    DebugBufferID { index: current_generated_index, buffer_type: self.buffer_type },
                    &self.collection_handle,
                );
                current_generated_index += 1;
                buf
            })
        }

        self.pool[index].clone()
    }
}

pub(crate) struct SharedBufferPool {
    pub audio_buffer_pool: BufferPool<f32>,
    pub note_buffer_pool: BufferPool<NoteIoEvent>,
    pub automation_buffer_pool: BufferPool<AutomationIoEvent>,
}

impl SharedBufferPool {
    pub fn new(
        audio_buffer_size: usize,
        note_buffer_size: usize,
        event_buffer_size: usize,
        coll_handle: basedrop::Handle,
    ) -> Self {
        Self {
            audio_buffer_pool: BufferPool::new(
                audio_buffer_size,
                DebugBufferType::Audio32,
                coll_handle.clone(),
            ),
            note_buffer_pool: BufferPool::new(
                note_buffer_size,
                DebugBufferType::Note,
                coll_handle.clone(),
            ),
            automation_buffer_pool: BufferPool::new(
                event_buffer_size,
                DebugBufferType::Event,
                coll_handle,
            ),
        }
    }

    pub fn set_num_buffers(
        &mut self,
        num_audio_buffers: usize,
        num_note_buffers: usize,
        num_automation_buffers: usize,
    ) {
        if num_audio_buffers > self.audio_buffer_pool.pool.len() {
            // Cause the pool to allocate enough slots.
            let _ = self.audio_buffer_pool.initialized_buffer_at_index(num_audio_buffers - 1);
        }
        if num_note_buffers > self.note_buffer_pool.pool.len() {
            // Cause the pool to allocate enough slots.
            let _ = self.note_buffer_pool.buffer_at_index(num_note_buffers - 1);
        }
        if num_automation_buffers > self.automation_buffer_pool.pool.len() {
            // Cause the pool to allocate enough slots.
            let _ = self.automation_buffer_pool.buffer_at_index(num_automation_buffers - 1);
        }

        self.audio_buffer_pool.pool.truncate(num_audio_buffers);
        self.note_buffer_pool.pool.truncate(num_note_buffers);
        self.automation_buffer_pool.pool.truncate(num_automation_buffers);
    }
}
