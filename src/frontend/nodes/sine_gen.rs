use basedrop::{Handle, Shared, SharedCell};

use crate::shared_state::{AudioGraphNode, ProcInfo};

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
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_out_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _audio_through: &mut Vec<Shared<Vec<f32>>>,
        _extra_audio_in: &Vec<Shared<Vec<f32>>>,
        extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        let model = *self.model.get();

        // This should not panic because the rt thread is the only place these buffers
        // are mutated.
        //
        // TODO: Find a way to do this more ergonomically and efficiently, perhaps by
        // using a safe wrapper around a custom type? We also want to make it so
        // a buffer can never be resized.
        let (left_out, right_out) = extra_audio_out.split_first_mut().unwrap();
        let left_out = Shared::get_mut(left_out).unwrap();
        let right_out = Shared::get_mut(&mut right_out[1]).unwrap();

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
                *left_out.get_unchecked_mut(i) = smp;
                *right_out.get_unchecked_mut(i) = smp;
            }
        }
    }
}
