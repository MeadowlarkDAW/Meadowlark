use super::{AudioGraphNode, ProcInfo};

// TODO: Smooth parameters. We can take inspiration from baseplug to create a system
// which automatically smooths parameters for us.

pub struct MonoGainNode {
    pub gain: f32,
}

impl MonoGainNode {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

impl Default for MonoGainNode {
    fn default() -> Self {
        MonoGainNode::new(1.0)
    }
}

impl AudioGraphNode for MonoGainNode {
    fn audio_through_ports(&self) -> usize {
        1
    }

    fn process(&mut self, proc_info: ProcInfo) {
        for smp in proc_info.audio_through[0].iter_mut() {
            *smp *= self.gain;
        }
    }
}

pub struct StereoGainNode {
    pub gain: f32,
}

impl StereoGainNode {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

impl Default for StereoGainNode {
    fn default() -> Self {
        StereoGainNode::new(1.0)
    }
}

impl AudioGraphNode for StereoGainNode {
    fn audio_through_ports(&self) -> usize {
        2
    }

    fn process(&mut self, proc_info: ProcInfo) {
        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures this validity.
            //
            // TODO: Find a more ergonomic way to do this using a custom type?
            unsafe {
                *proc_info
                    .extra_audio_out
                    .get_unchecked_mut(0)
                    .get_unchecked_mut(i) *= self.gain;
                *proc_info
                    .extra_audio_out
                    .get_unchecked_mut(1)
                    .get_unchecked_mut(i) *= self.gain;
            }
        }
    }
}
