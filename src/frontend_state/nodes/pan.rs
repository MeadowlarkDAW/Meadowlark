use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::frontend_state::{Gradient, ParamF32, ParamF32Handle, Unit};
use crate::graph_state::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer, MAX_BLOCKSIZE,
};

use super::{DB_GRADIENT, SMOOTH_MS};

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
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, StereoGainPanHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_MS,
            sample_rate,
            coll_handle.clone(),
        );

        let (pan, pan_handle) = ParamF32::from_value(
            pan,
            0.0,
            1.0,
            Gradient::Linear,
            Unit::Generic,
            SMOOTH_MS,
            sample_rate,
            coll_handle,
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
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        // TODO: Proper pan laws.

        // TODO: Optimize with SIMD.

        let gain_amp = self.gain_amp.smoothed(proc_info.frames);
        let pan = self.pan.smoothed(proc_info.frames);

        let src = &stereo_audio_in[0];
        let dst = &mut stereo_audio_out[0];

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

        if pan.is_smoothing() {
            // Need to calculate left and right gain per sample.
            match self.pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.

                    for i in 0..frames {
                        dst.left[i] = src.left[i] * (1.0 - pan.values[i]) * gain_amp.values[i];
                        dst.right[i] = src.right[i] * pan.values[i] * gain_amp.values[i];
                    }
                }
            }
        } else {
            // We can optimize by only calculating left and right gain once.
            let (left_amp, right_amp) = match self.pan_law {
                PanLaw::Linear => {
                    // TODO: I'm not sure this is actually linear pan-law. I'm just getting something down for now.

                    (1.0 - pan.values[0], pan.values[0])
                }
            };

            if gain_amp.is_smoothing() {
                for i in 0..frames {
                    dst.left[i] = src.right[i] * left_amp * gain_amp.values[i];
                    dst.right[i] = src.right[i] * right_amp * gain_amp.values[i];
                }
            } else {
                // We can optimize by pre-multiplying gain to the pan.
                let left_amp = left_amp * gain_amp.values[0];
                let right_amp = right_amp * gain_amp.values[0];

                for i in 0..frames {
                    dst.left[i] = src.right[i] * left_amp;
                    dst.right[i] = src.right[i] * right_amp;
                }
            }
        }
    }
}
