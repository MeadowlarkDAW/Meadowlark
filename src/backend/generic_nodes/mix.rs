use atomic_refcell::{AtomicRef, AtomicRefMut};

use crate::backend::audio_graph::{
    AudioGraphNode, MonoAudioBlockBuffer, ProcInfo, StereoAudioBlockBuffer,
};
use crate::backend::timeline::TimelineTransport;

pub struct MonoMixNode {
    num_inputs: usize,
}

impl MonoMixNode {
    /// Must have at-least two inputs.
    ///
    /// If `num_inputs < 2`, then two inputs will be created anyway.
    pub fn new(num_inputs: usize) -> Self {
        let num_inputs = num_inputs.max(2);
        Self { num_inputs }
    }
}

impl AudioGraphNode for MonoMixNode {
    fn mono_audio_in_ports(&self) -> usize {
        self.num_inputs
    }
    fn mono_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        mono_audio_in: &[AtomicRef<MonoAudioBlockBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioBlockBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioBlockBuffer>],
        _stereo_audio_out: &mut [AtomicRefMut<StereoAudioBlockBuffer>],
    ) {
        let dst = &mut mono_audio_out[0];

        // TODO: Optimize this.

        for i in 0..proc_info.frames() {
            // Safe because the scheduler upholds that there will always be `self.num_inputs` input
            // buffers, and there are always at-least two inputs.
            dst.buf[i] = unsafe { mono_audio_in.get_unchecked(0).buf[i] };
            for ch in 1..self.num_inputs {
                // Safe because the scheduler upholds that there will always be `self.num_inputs` input
                // buffers.
                dst.buf[i] += unsafe { mono_audio_in.get_unchecked(ch).buf[i] };
            }
        }
    }
}

pub struct StereoMixNode {
    num_inputs: usize,
}

impl StereoMixNode {
    /// Must have at-least two inputs.
    ///
    /// If `num_inputs < 2`, then two inputs will be created anyway.
    pub fn new(num_inputs: usize) -> Self {
        let num_inputs = num_inputs.max(2);
        Self { num_inputs }
    }
}

impl AudioGraphNode for StereoMixNode {
    fn stereo_audio_in_ports(&self) -> usize {
        self.num_inputs
    }
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        _mono_audio_in: &[AtomicRef<MonoAudioBlockBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioBlockBuffer>],
        stereo_audio_in: &[AtomicRef<StereoAudioBlockBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioBlockBuffer>],
    ) {
        let dst = &mut stereo_audio_out[0];

        // TODO: Optimize this with SIMD.

        for i in 0..proc_info.frames() {
            // Safe because the scheduler upholds that there will always be `self.num_inputs` input
            // buffers, and there are always at-least two inputs.
            unsafe {
                dst.left[i] = stereo_audio_in.get_unchecked(0).left[i];
                dst.right[i] = stereo_audio_in.get_unchecked(0).right[i];
            }
            for ch in 1..self.num_inputs {
                // Safe because the scheduler upholds that there will always be `self.num_inputs` input
                // buffers.
                unsafe {
                    dst.left[i] += stereo_audio_in.get_unchecked(ch).left[i];
                    dst.right[i] += stereo_audio_in.get_unchecked(ch).right[i];
                }
            }
        }
    }
}
