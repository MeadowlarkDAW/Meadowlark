use super::{AudioGraphNode, ProcInfo};

// TODO: Smooth parameters. We can take inspiration from baseplug to create a system
// which automatically smooths parameters for us.

pub struct StereoSineGenNode {
    pub pitch: f32,
    pub amp: f32,

    sample_clock: f32,
}

impl StereoSineGenNode {
    pub fn new(pitch: f32, amp: f32) -> Self {
        Self {
            pitch,
            amp,
            sample_clock: 0.0,
        }
    }
}

impl Default for StereoSineGenNode {
    fn default() -> Self {
        StereoSineGenNode::new(440.0, 1.0)
    }
}

impl AudioGraphNode for StereoSineGenNode {
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_out_ports(&self) -> usize {
        2
    }

    fn process(&mut self, proc_info: ProcInfo) {
        let period = 2.0 * std::f32::consts::PI * proc_info.sample_rate_recip;
        for i in 0..proc_info.frames {
            // TODO: This algorithm could be optimized.

            self.sample_clock = (self.sample_clock + 1.0) % proc_info.sample_rate;
            let smp = (self.sample_clock * self.pitch * period).sin() * self.amp;

            // Safe because the scheduler calling this method ensures this validity.
            //
            // TODO: Find a more ergonomic way to do this using a custom type?
            unsafe {
                *proc_info
                    .extra_audio_out
                    .get_unchecked_mut(0)
                    .get_unchecked_mut(i) = smp;
                *proc_info
                    .extra_audio_out
                    .get_unchecked_mut(1)
                    .get_unchecked_mut(i) = smp;
            }
        }
    }
}
