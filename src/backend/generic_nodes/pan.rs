use atomic_refcell::{AtomicRef, AtomicRefMut};
use rusty_daw_time::SampleRate;

use crate::backend::audio_graph::{
    AudioGraphNode, MonoAudioBlockBuffer, ProcInfo, StereoAudioBlockBuffer,
};
use crate::backend::cpu_id;
use crate::backend::parameter::{Gradient, ParamF32, ParamF32Handle, Unit};
use crate::backend::timeline::TimelineTransport;

use super::{DB_GRADIENT, SMOOTH_SECS};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanLaw {
    Linear,
}

pub struct StereoGainPanHandle {
    pub gain_db: ParamF32Handle,
    pub pan: ParamF32Handle,

    pan_law: PanLaw,
}

impl StereoGainPanHandle {
    pub fn pan_law(&self) -> &PanLaw {
        &self.pan_law
    }
}

pub struct StereoGainPanNode {
    gain_amp: ParamF32,
    pan: ParamF32,
    pan_law: PanLaw,
}

impl StereoGainPanNode {
    pub fn new(
        gain_db: f32,
        min_db: f32,
        max_db: f32,
        pan: f32,
        pan_law: PanLaw,
        sample_rate: SampleRate,
    ) -> (Self, StereoGainPanHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        let (pan, pan_handle) = ParamF32::from_value(
            pan,
            0.0,
            1.0,
            Gradient::Linear,
            Unit::Generic,
            SMOOTH_SECS,
            sample_rate,
        );

        (
            Self {
                gain_amp,
                pan,
                pan_law,
            },
            StereoGainPanHandle {
                gain_db: gain_handle,
                pan: pan_handle,
                pan_law,
            },
        )
    }
}

impl AudioGraphNode for StereoGainPanNode {
    fn stereo_audio_in_ports(&self) -> usize {
        1
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
        let gain_amp = self.gain_amp.smoothed(proc_info.frames());
        let pan = self.pan.smoothed(proc_info.frames());

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::stereo_gain_pan_avx(
                        proc_info,
                        &gain_amp,
                        &pan,
                        self.pan_law,
                        &stereo_audio_in[0],
                        &mut stereo_audio_out[0],
                    );
                }
                return;
            }
        }

        simd::stereo_gain_pan_fallback(
            proc_info,
            &gain_amp,
            &pan,
            self.pan_law,
            &stereo_audio_in[0],
            &mut stereo_audio_out[0],
        );
    }
}

mod simd {
    use super::{PanLaw, StereoAudioBlockBuffer};
    use crate::backend::cpu_id;
    use crate::backend::{audio_graph::ProcInfo, parameter::SmoothOutput};

    pub fn stereo_gain_pan_fallback(
        proc_info: &ProcInfo,
        gain_amp: &SmoothOutput<f32>,
        pan: &SmoothOutput<f32>,
        pan_law: PanLaw,
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
    ) {
        let frames = proc_info.frames();

        if pan.is_smoothing() {
            // Need to calculate left and right gain per sample.
            match pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.

                    if gain_amp.is_smoothing() {
                        for i in 0..frames {
                            dst.left[i] = src.left[i] * (1.0 - pan.values[i]) * gain_amp.values[i];
                            dst.right[i] = src.right[i] * pan.values[i] * gain_amp.values[i];
                        }
                    } else {
                        // We can optimize by using a constant gain (better SIMD load efficiency).
                        let gain = gain_amp.values[0];

                        for i in 0..frames {
                            dst.left[i] = src.left[i] * (1.0 - pan.values[i]) * gain;
                            dst.right[i] = src.right[i] * pan.values[i] * gain;
                        }
                    }
                }
            }
        } else {
            // We can optimize by only calculating left and right gain once.
            let (left_amp, right_amp) = match pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.
                    (1.0 - pan.values[0], pan.values[0])
                }
            };

            if gain_amp.is_smoothing() {
                for i in 0..frames {
                    dst.left[i] = src.left[i] * left_amp * gain_amp.values[i];
                    dst.right[i] = src.right[i] * right_amp * gain_amp.values[i];
                }
            } else {
                // We can optimize by pre-multiplying gain to the pan.
                let left_amp = left_amp * gain_amp.values[0];
                let right_amp = right_amp * gain_amp.values[0];

                for i in 0..frames {
                    dst.left[i] = src.left[i] * left_amp;
                    dst.right[i] = src.right[i] * right_amp;
                }
            }
        }
    }

    // Since stereo gain and pan is so common, it makes sense to make it as optimized as possible.
    // So manual SIMD is used here.
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn stereo_gain_pan_avx(
        proc_info: &ProcInfo,
        gain_amp: &SmoothOutput<f32>,
        pan: &SmoothOutput<f32>,
        pan_law: PanLaw,
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let frames = proc_info.frames();

        if pan.is_smoothing() {
            // Need to calculate left and right gain per sample.
            match pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.

                    let one_v = _mm256_set1_ps(1.0);

                    if gain_amp.is_smoothing() {
                        // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
                        //
                        // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
                        // is more efficient to process it as a block anyway. It doesn't matter if the last block
                        // contains uninitialized data because we won't read it anyway.
                        for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                            let src_left_v = _mm256_loadu_ps(&src.left[i]);
                            let src_right_v = _mm256_loadu_ps(&src.right[i]);

                            let gain_v = _mm256_loadu_ps(&gain_amp.values[i]);
                            let right_pan_v = _mm256_loadu_ps(&pan.values[i]);

                            let left_pan_v = _mm256_sub_ps(one_v, right_pan_v);

                            let mul_left_v =
                                _mm256_mul_ps(_mm256_mul_ps(src_left_v, left_pan_v), gain_v);
                            let mul_right_v =
                                _mm256_mul_ps(_mm256_mul_ps(src_right_v, right_pan_v), gain_v);

                            _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                            _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
                        }
                    } else {
                        // We can optimize by using a constant gain (better SIMD load efficiency).
                        let gain_v = _mm256_set1_ps(gain_amp.values[0]);

                        // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
                        //
                        // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
                        // is more efficient to process it as a block anyway. It doesn't matter if the last block
                        // contains uninitialized data because we won't read it anyway.
                        for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                            let src_left_v = _mm256_loadu_ps(&src.left[i]);
                            let src_right_v = _mm256_loadu_ps(&src.right[i]);

                            let right_pan_v = _mm256_loadu_ps(&pan.values[i]);

                            let left_pan_v = _mm256_sub_ps(one_v, right_pan_v);

                            let mul_left_v =
                                _mm256_mul_ps(_mm256_mul_ps(src_left_v, left_pan_v), gain_v);
                            let mul_right_v =
                                _mm256_mul_ps(_mm256_mul_ps(src_right_v, right_pan_v), gain_v);

                            _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                            _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
                        }
                    }
                }
            }
        } else {
            // We can optimize by only calculating left and right gain once.
            let (left_pan, right_pan) = match pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.
                    (1.0 - pan.values[0], pan.values[0])
                }
            };

            let left_pan_v = _mm256_set1_ps(left_pan);
            let right_pan_v = _mm256_set1_ps(right_pan);

            if gain_amp.is_smoothing() {
                // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
                //
                // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
                // is more efficient to process it as a block anyway. It doesn't matter if the last block
                // contains uninitialized data because we won't read it anyway.
                for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                    let src_left_v = _mm256_loadu_ps(&src.left[i]);
                    let src_right_v = _mm256_loadu_ps(&src.right[i]);

                    let gain_v = _mm256_loadu_ps(&gain_amp.values[i]);

                    let mul_left_v = _mm256_mul_ps(_mm256_mul_ps(src_left_v, left_pan_v), gain_v);
                    let mul_right_v =
                        _mm256_mul_ps(_mm256_mul_ps(src_right_v, right_pan_v), gain_v);

                    _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                    _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
                }
            } else {
                // We can optimize by pre-multiplying gain to the pan.
                let left_amp = left_pan * gain_amp.values[0];
                let right_amp = right_pan * gain_amp.values[0];

                let left_amp_v = _mm256_set1_ps(left_amp);
                let right_amp_v = _mm256_set1_ps(right_amp);

                // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
                //
                // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
                // is more efficient to process it as a block anyway. It doesn't matter if the last block
                // contains uninitialized data because we won't read it anyway.
                for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                    let src_left_v = _mm256_loadu_ps(&src.left[i]);
                    let src_right_v = _mm256_loadu_ps(&src.right[i]);

                    let mul_left_v = _mm256_mul_ps(src_left_v, left_amp_v);
                    let mul_right_v = _mm256_mul_ps(src_right_v, right_amp_v);

                    _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                    _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
                }
            }
        }
    }
}
