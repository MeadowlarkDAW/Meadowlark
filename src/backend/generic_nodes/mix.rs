use atomic_refcell::{AtomicRef, AtomicRefMut};

use crate::backend::cpu_id;
use crate::backend::graph::{
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
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::mono_mix_avx(proc_info, mono_audio_in, &mut mono_audio_out[0]);
                }
                return;
            }
        }

        simd::mono_mix_fallback(proc_info, mono_audio_in, &mut mono_audio_out[0]);
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
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::stereo_mix_avx(proc_info, stereo_audio_in, &mut stereo_audio_out[0]);
                }
                return;
            }
        }

        simd::stereo_mix_fallback(proc_info, stereo_audio_in, &mut stereo_audio_out[0]);
    }
}

mod simd {
    use atomic_refcell::AtomicRef;

    use super::{MonoAudioBlockBuffer, StereoAudioBlockBuffer};
    use crate::backend::graph::ProcInfo;

    pub fn mono_mix_fallback(
        proc_info: &ProcInfo,
        src: &[AtomicRef<MonoAudioBlockBuffer>],
        dst: &mut MonoAudioBlockBuffer,
    ) {
        // Hint to compiler to optimize loops.
        let frames = proc_info.frames();
        if src.is_empty() {
            return;
        }

        &mut dst.buf[0..frames].copy_from_slice(&src[0].buf[0..frames]);

        match src.len() {
            0 => return,
            1 => return,
            2 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i];
                }
            }
            3 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i];
                }
            }
            4 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i] + src[3].buf[i];
                }
            }
            5 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i] + src[3].buf[i] + src[4].buf[i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i];
                }
            }
            7 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i]
                        + src[6].buf[i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i]
                        + src[6].buf[i]
                        + src[7].buf[i];
                }
            }
            // TODO: We can goes as high as we expect a typical maximum in a project
            // will be.
            len => {
                // TODO: Benchmark whether using a match to a specialized loop is actually
                // any better than using this single catch-all expression:

                for i in 0..frames {
                    for ch in 1..len {
                        dst.buf[i] += src[ch].buf[i];
                    }
                }
            }
        }
    }

    // TODO: Find an elegant way to share code with the fallback method when relying
    // on auto-vectorization.
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn mono_mix_avx(
        proc_info: &ProcInfo,
        src: &[AtomicRef<MonoAudioBlockBuffer>],
        dst: &mut MonoAudioBlockBuffer,
    ) {
        // Hint to compiler to optimize loops.
        let frames = proc_info.frames();
        if src.is_empty() {
            return;
        }

        &mut dst.buf[0..frames].copy_from_slice(&src[0].buf[0..frames]);

        match src.len() {
            0 => return,
            1 => return,
            2 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i];
                }
            }
            3 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i];
                }
            }
            4 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i] + src[3].buf[i];
                }
            }
            5 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i] + src[2].buf[i] + src[3].buf[i] + src[4].buf[i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i];
                }
            }
            7 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i]
                        + src[6].buf[i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst.buf[i] += src[1].buf[i]
                        + src[2].buf[i]
                        + src[3].buf[i]
                        + src[4].buf[i]
                        + src[5].buf[i]
                        + src[6].buf[i]
                        + src[7].buf[i];
                }
            }
            // TODO: We can goes as high as we expect a typical maximum in a project
            // will be.
            len => {
                // TODO: Benchmark whether using a match to a specialized loop is actually
                // any better than using this single catch-all expression:

                for i in 0..frames {
                    for ch in 1..len {
                        dst.buf[i] += src[ch].buf[i];
                    }
                }
            }
        }
    }

    pub fn stereo_mix_fallback(
        proc_info: &ProcInfo,
        src: &[AtomicRef<StereoAudioBlockBuffer>],
        dst: &mut StereoAudioBlockBuffer,
    ) {
        // Hint to compiler to optimize loops.
        let frames = proc_info.frames();
        if src.is_empty() {
            return;
        }

        &mut dst.left[0..frames].copy_from_slice(&src[0].left[0..frames]);
        &mut dst.right[0..frames].copy_from_slice(&src[0].right[0..frames]);

        match src.len() {
            0 => return,
            1 => return,
            2 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i];
                    dst.right[i] += src[1].right[i];
                }
            }
            3 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i] + src[2].left[i];
                    dst.right[i] += src[1].right[i] + src[2].right[i];
                }
            }
            4 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i] + src[2].left[i] + src[3].left[i];
                    dst.right[i] += src[1].right[i] + src[2].right[i] + src[3].right[i];
                }
            }
            5 => {
                for i in 0..frames {
                    dst.left[i] +=
                        src[1].left[i] + src[2].left[i] + src[3].left[i] + src[4].left[i];
                    dst.right[i] +=
                        src[1].right[i] + src[2].right[i] + src[3].right[i] + src[4].right[i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i];
                }
            }
            7 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i]
                        + src[6].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i]
                        + src[6].right[i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i]
                        + src[6].left[i]
                        + src[7].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i]
                        + src[6].right[i]
                        + src[7].right[i];
                }
            }
            // TODO: We can goes as high as we expect a typical maximum in a project
            // will be.
            len => {
                // TODO: Benchmark whether using a match to a specialized loop is actually
                // any better than using this single catch-all expression:

                for i in 0..frames {
                    for ch in 1..len {
                        dst.left[i] += src[ch].left[i];
                        dst.right[i] += src[ch].right[i];
                    }
                }
            }
        }
    }

    // TODO: Find an elegant way to share code with the fallback method when relying
    // on auto-vectorization.
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn stereo_mix_avx(
        proc_info: &ProcInfo,
        src: &[AtomicRef<StereoAudioBlockBuffer>],
        dst: &mut StereoAudioBlockBuffer,
    ) {
        // Hint to compiler to optimize loops.
        let frames = proc_info.frames();
        if src.is_empty() {
            return;
        }

        &mut dst.left[0..frames].copy_from_slice(&src[0].left[0..frames]);
        &mut dst.right[0..frames].copy_from_slice(&src[0].right[0..frames]);

        match src.len() {
            0 => return,
            1 => return,
            2 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i];
                    dst.right[i] += src[1].right[i];
                }
            }
            3 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i] + src[2].left[i];
                    dst.right[i] += src[1].right[i] + src[2].right[i];
                }
            }
            4 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i] + src[2].left[i] + src[3].left[i];
                    dst.right[i] += src[1].right[i] + src[2].right[i] + src[3].right[i];
                }
            }
            5 => {
                for i in 0..frames {
                    dst.left[i] +=
                        src[1].left[i] + src[2].left[i] + src[3].left[i] + src[4].left[i];
                    dst.right[i] +=
                        src[1].right[i] + src[2].right[i] + src[3].right[i] + src[4].right[i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i];
                }
            }
            7 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i]
                        + src[6].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i]
                        + src[6].right[i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst.left[i] += src[1].left[i]
                        + src[2].left[i]
                        + src[3].left[i]
                        + src[4].left[i]
                        + src[5].left[i]
                        + src[6].left[i]
                        + src[7].left[i];
                    dst.right[i] += src[1].right[i]
                        + src[2].right[i]
                        + src[3].right[i]
                        + src[4].right[i]
                        + src[5].right[i]
                        + src[6].right[i]
                        + src[7].right[i];
                }
            }
            // TODO: We can goes as high as we expect a typical maximum in a project
            // will be.
            len => {
                // TODO: Benchmark whether using a match to a specialized loop is actually
                // any better than using this single catch-all expression:

                for i in 0..frames {
                    for ch in 1..len {
                        dst.left[i] += src[ch].left[i];
                        dst.right[i] += src[ch].right[i];
                    }
                }
            }
        }
    }
}
