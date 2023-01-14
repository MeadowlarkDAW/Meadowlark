// Modified code from: https://github.com/RustAudio/vst-rs/blob/master/src/util/atomic_float.rs
//
// The MIT License (MIT)
//
// Copyright (c) 2015 Marko Mijalkovic
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Simple atomic `f32` floating point variable with relaxed ordering.
pub struct AtomicF32 {
    atomic: AtomicU32,
}

impl AtomicF32 {
    /// New atomic float with initial value `value`.
    #[inline]
    pub fn new(value: f32) -> AtomicF32 {
        AtomicF32 { atomic: AtomicU32::new(value.to_bits()) }
    }

    /// Get the current value of the atomic float.
    #[inline]
    pub fn get(&self) -> f32 {
        f32::from_bits(self.atomic.load(Ordering::Relaxed))
    }

    /// Set the value of the atomic float to `value`.
    #[inline]
    pub fn set(&self, value: f32) {
        self.atomic.store(value.to_bits(), Ordering::Relaxed)
    }
}

impl Default for AtomicF32 {
    fn default() -> Self {
        AtomicF32::new(0.0)
    }
}

impl std::fmt::Debug for AtomicF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.get(), f)
    }
}

impl std::fmt::Display for AtomicF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.get(), f)
    }
}

impl From<f32> for AtomicF32 {
    fn from(value: f32) -> Self {
        AtomicF32::new(value)
    }
}

impl From<AtomicF32> for f32 {
    fn from(value: AtomicF32) -> Self {
        value.get()
    }
}

// ------  F64  -------------------------------------------------------------------------

/// Simple atomic `f64` floating point variable with relaxed ordering.
pub struct AtomicF64 {
    atomic: AtomicU64,
}

impl AtomicF64 {
    /// New atomic float with initial value `value`.
    #[inline]
    pub fn new(value: f64) -> AtomicF64 {
        AtomicF64 { atomic: AtomicU64::new(value.to_bits()) }
    }

    /// Get the current value of the atomic float.
    #[inline]
    pub fn get(&self) -> f64 {
        f64::from_bits(self.atomic.load(Ordering::Relaxed))
    }

    /// Set the value of the atomic float to `value`.
    #[inline]
    pub fn set(&self, value: f64) {
        self.atomic.store(value.to_bits(), Ordering::Relaxed)
    }
}

impl Default for AtomicF64 {
    fn default() -> Self {
        AtomicF64::new(0.0)
    }
}

impl std::fmt::Debug for AtomicF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.get(), f)
    }
}

impl std::fmt::Display for AtomicF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.get(), f)
    }
}

impl From<f64> for AtomicF64 {
    fn from(value: f64) -> Self {
        AtomicF64::new(value)
    }
}

impl From<AtomicF64> for f64 {
    fn from(value: AtomicF64) -> Self {
        value.get()
    }
}
