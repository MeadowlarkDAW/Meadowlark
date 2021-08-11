use std::mem::MaybeUninit;
use std::ops::Range;

use crate::backend::MAX_BLOCKSIZE;

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
    pub fn new() -> Self {
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
