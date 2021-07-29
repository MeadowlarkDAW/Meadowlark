use std::mem::MaybeUninit;
use std::ops::Range;

use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared};

use super::{node::AudioGraphNode, MAX_BLOCKSIZE};

#[derive(Clone)]
pub struct GraphResourcePool {
    // Using AtomicRefCell because these resources are only ever borrowed by
    // the rt thread. We keep these pointers in a non-rt thread so we can
    // cheaply clone and reconstruct a new schedule to send to the rt thread whenever the
    // graph is recompiled (only need to copy pointers instead of whole Vecs).
    pub(super) nodes: Vec<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>,
    pub(super) mono_audio_buffers: Vec<Shared<AtomicRefCell<MonoAudioBlockBuffer>>>,
    pub(super) stereo_audio_buffers: Vec<Shared<AtomicRefCell<StereoAudioBlockBuffer>>>,

    coll_handle: Handle,
}

/*
impl Clone for GraphResourcePool {
    fn clone(&self) -> Self {
        let mut nodes =
            Vec::<Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>>::with_capacity(self.nodes.len());
        let mut mono_audio_buffers =
            Vec::<Shared<AtomicRefCell<MonoAudioBlockBuffer>>>::with_capacity(
                self.mono_audio_buffers.len(),
            );
        let mut stereo_audio_buffers =
            Vec::<Shared<AtomicRefCell<StereoAudioBlockBuffer>>>::with_capacity(
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
*/

impl GraphResourcePool {
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

    /// Replaces a node in the pool. Only to be used by the non-rt thread.
    pub(super) fn replace_node(
        &mut self,
        node_index: usize,
        new_node: Box<dyn AudioGraphNode>,
    ) -> Result<(), ()> {
        if node_index < self.nodes.len() {
            self.nodes[node_index] = Shared::new(&self.coll_handle, AtomicRefCell::new(new_node));
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
                AtomicRefCell::new(MonoAudioBlockBuffer::new()),
            ));
        }
    }

    /// Add new stereo audio port buffer to the pool. Only to be used by the non-rt thread.
    pub(super) fn add_stereo_audio_port_buffers(&mut self, n_new_port_buffers: usize) {
        for _ in 0..n_new_port_buffers {
            self.stereo_audio_buffers.push(Shared::new(
                &self.coll_handle,
                AtomicRefCell::new(StereoAudioBlockBuffer::new()),
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

/// An audio buffer with a single channel.
///
/// This has a constant number of frames (`MAX_BLOCKSIZE`), so this can be allocated on
/// the stack.
#[derive(Debug)]
pub struct MonoAudioBlockBuffer {
    pub buf: [f32; MAX_BLOCKSIZE],
}

impl MonoAudioBlockBuffer {
    /// Create a new buffer.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// All samples will be cleared to 0.
    fn new() -> Self {
        Self {
            buf: [0.0; MAX_BLOCKSIZE],
        }
    }

    /// Create a new buffer without initializing.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// This data will be unitialized, so undefined behavior may occur if you try to read
    /// any data without writing to it first.
    pub unsafe fn new_uninit() -> Self {
        Self {
            buf: MaybeUninit::uninit().assume_init(),
        }
    }

    /// Create a new buffer that only initializes the given range of data to 0.0.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// The portion of data not in the given range will be unitialized, so undefined behavior
    /// may occur if you try to read any of that data without writing to it first.
    ///
    /// ## Panics
    /// This will panic if the given range lies outside the valid range `[0, MAX_BLOCKSIZE)`.
    pub unsafe fn new_partially_uninit(init_range: Range<usize>) -> Self {
        let mut buf: [f32; MAX_BLOCKSIZE] = MaybeUninit::uninit().assume_init();

        let buf_part = &mut buf[init_range];
        buf_part.fill(0.0);

        Self { buf }
    }

    /// Clear all samples in the buffer to 0.0.
    pub fn clear(&mut self) {
        self.buf.fill(0.0);
    }

    pub fn copy_from(&mut self, src: &MonoAudioBlockBuffer) {
        self.buf.copy_from_slice(&src.buf);
    }

    pub fn copy_from_stereo_left(&mut self, src: &StereoAudioBlockBuffer) {
        self.buf.copy_from_slice(&src.left);
    }
    pub fn copy_from_stereo_right(&mut self, src: &StereoAudioBlockBuffer) {
        self.buf.copy_from_slice(&src.right);
    }
}

/// An audio buffer with two channels (left and right).
///
/// This has a constant number of frames (`MAX_BLOCKSIZE`), so this can be allocated on
/// the stack.
#[derive(Debug)]
pub struct StereoAudioBlockBuffer {
    pub left: [f32; MAX_BLOCKSIZE],
    pub right: [f32; MAX_BLOCKSIZE],
}

impl StereoAudioBlockBuffer {
    /// Create a new buffer.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// All samples will be cleared to 0.
    pub fn new() -> Self {
        Self {
            left: [0.0; MAX_BLOCKSIZE],
            right: [0.0; MAX_BLOCKSIZE],
        }
    }

    /// Create a new buffer without initializing.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// This data will be unitialized, so undefined behavior may occur if you try to read
    /// any data without writing to it first.
    pub unsafe fn new_uninit() -> Self {
        Self {
            left: MaybeUninit::uninit().assume_init(),
            right: MaybeUninit::uninit().assume_init(),
        }
    }

    /// Create a new buffer that only initializes the given range of data to 0.0.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// The portion of data not in the given range will be unitialized, so undefined behavior
    /// may occur if you try to read any of that data without writing to it first.
    ///
    /// ## Panics
    /// This will panic if the given range lies outside the valid range `[0, MAX_BLOCKSIZE)`.
    pub unsafe fn new_partially_uninit(init_range: Range<usize>) -> Self {
        let mut left: [f32; MAX_BLOCKSIZE] = MaybeUninit::uninit().assume_init();
        let mut right: [f32; MAX_BLOCKSIZE] = MaybeUninit::uninit().assume_init();

        let left_part = &mut left[init_range.clone()];
        let right_part = &mut right[init_range];
        left_part.fill(0.0);
        right_part.fill(0.0);

        Self { left, right }
    }

    /// Clear all samples in both channels to 0.0.
    pub fn clear(&mut self) {
        self.left.fill(0.0);
        self.right.fill(0.0);
    }

    /// Clear all samples in the left channel to 0.0.
    pub fn clear_left(&mut self) {
        self.left.fill(0.0);
    }

    /// Clear all samples in the right channel to 0.0.
    pub fn clear_right(&mut self) {
        self.right.fill(0.0);
    }

    pub fn copy_from(&mut self, src: &StereoAudioBlockBuffer) {
        self.left.copy_from_slice(&src.left);
        self.right.copy_from_slice(&src.right);
    }

    pub fn copy_from_mono_to_left(&mut self, src: &MonoAudioBlockBuffer) {
        self.left.copy_from_slice(&src.buf);
    }
    pub fn copy_from_mono_to_right(&mut self, src: &MonoAudioBlockBuffer) {
        self.right.copy_from_slice(&src.buf);
    }
}
