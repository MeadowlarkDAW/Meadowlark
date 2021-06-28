use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};

use crate::graph_state::{AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer};

// TODO: Smooth parameters. We can take inspiration from baseplug to create a system
// which automatically smooths parameters for us.

#[derive(Debug, Clone, Copy)]
struct Model {
    pitch: f32,
    amp: f32,
}

pub struct StereoSineGenNodeHandle {
    model: Shared<SharedCell<Model>>,
    coll_handle: Handle,
}

impl StereoSineGenNodeHandle {
    pub fn pitch(&self) -> f32 {
        self.model.get().pitch
    }
    pub fn amp(&self) -> f32 {
        self.model.get().pitch
    }

    pub fn set_params(&mut self, pitch: f32, amp: f32) {
        self.model
            .set(Shared::new(&self.coll_handle, Model { pitch, amp }));
    }
}

pub struct StereoSineGenNode {
    model: Shared<SharedCell<Model>>,

    sample_clock: f32,
}

impl StereoSineGenNode {
    pub fn new(pitch: f32, amp: f32, coll_handle: &Handle) -> (Self, StereoSineGenNodeHandle) {
        let model = Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(coll_handle, Model { pitch, amp })),
        );

        (
            Self {
                model: model.clone(),
                sample_clock: 0.0,
            },
            StereoSineGenNodeHandle {
                model,
                coll_handle: coll_handle.clone(),
            },
        )
    }
}

impl AudioGraphNode for StereoSineGenNode {
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let model = *self.model.get();

        let (dst_l, dst_r) = stereo_audio_out[0].left_right_mut();

        let period = 2.0 * std::f32::consts::PI * proc_info.sample_rate_recip;
        for i in 0..proc_info.frames {
            // TODO: This algorithm could be optimized.

            self.sample_clock = (self.sample_clock + 1.0) % proc_info.sample_rate;
            let smp = (self.sample_clock * model.pitch * period).sin() * model.amp;

            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            //
            // TODO: Find a more ergonomic way to do this using a safe wrapper around a
            // custom type? We also want to make it so
            // a buffer can never be resized.
            unsafe {
                *dst_l.get_unchecked_mut(i) = smp;
                *dst_r.get_unchecked_mut(i) = smp;
            }
        }
    }
}
