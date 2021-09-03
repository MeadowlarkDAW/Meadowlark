use std::mem::MaybeUninit;
use std::ops::Range;

use num_traits::Num;

use crate::backend::MAX_BLOCKSIZE;

/// An audio buffer with a single channel.
///
/// This has a constant number of frames (`MAX_BLOCKSIZE`), so this can be allocated on
/// the stack.
#[derive(Debug)]
pub struct AudioBlockBuffer<T: Num + Copy + Clone> {
    pub buf: [T; MAX_BLOCKSIZE],
}

impl<T: Num + Copy + Clone> AudioBlockBuffer<T> {
    /// Create a new buffer.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// All samples will be cleared to 0.
    pub fn new() -> Self {
        Self { buf: [T::zero(); MAX_BLOCKSIZE] }
    }

    /// Create a new buffer without initializing.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// This data will be unitialized, so undefined behavior may occur if you try to read
    /// any data without writing to it first.
    pub unsafe fn new_uninit() -> Self {
        Self { buf: MaybeUninit::uninit().assume_init() }
    }

    /// Create a new buffer that only initializes the given number of frames to 0. Any samples
    /// after `frames` will be uninitialized.
    ///
    /// This is a constant size (`MAX_BLOCKSIZE`), so this can be allocated on the stack.
    ///
    /// ## Undefined behavior
    /// The portion of data not in the given range will be unitialized, so undefined behavior
    /// may occur if you try to read any of that data without writing to it first.
    pub unsafe fn new_uninit_after_frames(frames: usize) -> Self {
        let frames = frames.min(MAX_BLOCKSIZE);
        let mut buf: [T; MAX_BLOCKSIZE] = MaybeUninit::uninit().assume_init();

        let buf_part = &mut buf[0..frames];
        buf_part.fill(T::zero());

        Self { buf }
    }

    /// Create a new buffer that only initializes the given range of data to 0.
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
        let mut buf: [T; MAX_BLOCKSIZE] = MaybeUninit::uninit().assume_init();

        let buf_part = &mut buf[init_range];
        buf_part.fill(T::zero());

        Self { buf }
    }

    /// Clear all samples in the buffer to 0.
    #[inline]
    pub fn clear(&mut self) {
        self.buf.fill(T::zero());
    }

    /// Clear a number of frames in the buffer to 0.
    #[inline]
    pub fn clear_frames(&mut self, frames: usize) {
        let frames = frames.min(MAX_BLOCKSIZE);
        let buf_part = &mut self.buf[0..frames];
        buf_part.fill(T::zero());
    }

    /// Clear a range in the buffer to 0.
    ///
    /// ## Panics
    /// This will panic if the given range lies outside the valid range `[0, MAX_BLOCKSIZE)`.
    #[inline]
    pub fn clear_range(&mut self, range: Range<usize>) {
        let buf_part = &mut self.buf[range];
        buf_part.fill(T::zero());
    }

    /// Copy all frames from `src` to this buffer.
    #[inline]
    pub fn copy_from(&mut self, src: &AudioBlockBuffer<T>) {
        self.buf.copy_from_slice(&src.buf);
    }

    /// Copy the given number of `frames` from `src` to this buffer.
    #[inline]
    pub fn copy_frames_from(&mut self, src: &AudioBlockBuffer<T>, frames: usize) {
        let frames = frames.min(MAX_BLOCKSIZE);
        &mut self.buf[0..frames].copy_from_slice(&src.buf[0..frames]);
    }

    /// Add all frames from `src` to this buffer.
    #[inline]
    pub fn sum_from(&mut self, src: &AudioBlockBuffer<T>) {
        for i in 0..MAX_BLOCKSIZE {
            self.buf[i] = self.buf[i] + src.buf[i];
        }
    }

    /// Add the given number of frames from `src` to this buffer.
    #[inline]
    pub fn sum_frames_from(&mut self, src: &AudioBlockBuffer<T>, frames: usize) {
        let frames = frames.min(MAX_BLOCKSIZE);
        for i in 0..frames {
            self.buf[i] = self.buf[i] + src.buf[i];
        }
    }

    /// Multiplay all frames from `src` to this buffer.
    #[inline]
    pub fn multiply_from(&mut self, src: &AudioBlockBuffer<T>) {
        for i in 0..MAX_BLOCKSIZE {
            self.buf[i] = self.buf[i] * src.buf[i];
        }
    }

    /// Multiply the given number of frames from `src` to this buffer.
    #[inline]
    pub fn multiply_frames_from(&mut self, src: &AudioBlockBuffer<T>, frames: usize) {
        let frames = frames.min(MAX_BLOCKSIZE);
        for i in 0..frames {
            self.buf[i] = self.buf[i] * src.buf[i];
        }
    }
}

impl<T, I> std::ops::Index<I> for AudioBlockBuffer<T>
where
    I: std::slice::SliceIndex<[T]>,
    T: Num + Copy + Clone,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, idx: I) -> &I::Output {
        &self.buf[idx]
    }
}
