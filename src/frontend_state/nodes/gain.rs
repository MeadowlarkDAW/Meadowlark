use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};

use crate::graph_state::{AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer};

// TODO: Smooth parameters. We can take inspiration from baseplug to create a system
// which automatically smooths parameters for us.

pub struct MonoGainNodeHandle {
    gain: Shared<SharedCell<f32>>,
    coll_handle: Handle,
}

impl MonoGainNodeHandle {
    pub fn gain(&self) -> f32 {
        *self.gain.get()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain.set(Shared::new(&self.coll_handle, gain));
    }
}

pub struct MonoGainNode {
    gain: Shared<SharedCell<f32>>,
}

impl MonoGainNode {
    pub fn new(gain: f32, coll_handle: &Handle) -> (Self, MonoGainNodeHandle) {
        let gain = Shared::new(coll_handle, SharedCell::new(Shared::new(coll_handle, gain)));

        (
            Self {
                gain: Shared::clone(&gain),
            },
            MonoGainNodeHandle {
                gain,
                coll_handle: coll_handle.clone(),
            },
        )
    }
}

impl AudioGraphNode for MonoGainNode {
    fn mono_audio_in_ports(&self) -> usize {
        1
    }
    fn mono_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        _stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain = *self.gain.get();

        // TODO: Manual SIMD (to take advantage of AVX)

        let src = mono_audio_in[0].get();
        let dst = mono_audio_out[0].get_mut();

        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst.get_unchecked_mut(i) = *src.get_unchecked(i) * gain;
            }
        }
    }
}

pub struct StereoGainNodeHandle {
    gain: Shared<SharedCell<f32>>,
    coll_handle: Handle,
}

impl StereoGainNodeHandle {
    pub fn gain(&self) -> f32 {
        *self.gain.get()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain.set(Shared::new(&self.coll_handle, gain));
    }
}

pub struct StereoGainNode {
    gain: Shared<SharedCell<f32>>,
}

impl StereoGainNode {
    pub fn new(gain: f32, coll_handle: &Handle) -> (Self, StereoGainNodeHandle) {
        let gain = Shared::new(coll_handle, SharedCell::new(Shared::new(coll_handle, gain)));

        (
            Self {
                gain: Shared::clone(&gain),
            },
            StereoGainNodeHandle {
                gain,
                coll_handle: coll_handle.clone(),
            },
        )
    }
}

impl AudioGraphNode for StereoGainNode {
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
        let gain = *self.gain.get();

        // TODO: Manual SIMD (to take advantage of AVX)

        let (src_l, src_r) = stereo_audio_in[0].left_right();
        let (dst_l, dst_r) = stereo_audio_out[0].left_right_mut();

        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst_l.get_unchecked_mut(i) = *src_l.get_unchecked(i) * gain;
                *dst_r.get_unchecked_mut(i) = *src_r.get_unchecked(i) * gain;
            }
        }
    }
}
