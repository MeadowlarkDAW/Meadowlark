use basedrop::{Handle, Shared, SharedCell};

use crate::shared_state::{AudioGraphNode, ProcInfo};

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
    fn audio_through_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        _proc_info: &ProcInfo,
        audio_through: &mut Vec<Shared<Vec<f32>>>,
        _extra_audio_in: &Vec<Shared<Vec<f32>>>,
        _extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        let gain = *self.gain.get();

        // This should not panic because the rt thread is the only place these buffers
        // are mutated.
        //
        // TODO: Find a way to do this more ergonomically and efficiently, perhaps by
        // using a safe wrapper around a custom type?
        for smp in Shared::get_mut(&mut audio_through[0]).unwrap().iter_mut() {
            *smp *= gain;
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
    fn audio_through_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        audio_through: &mut Vec<Shared<Vec<f32>>>,
        _extra_audio_in: &Vec<Shared<Vec<f32>>>,
        _extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        let gain = *self.gain.get();

        // This should not panic because the rt thread is the only place these buffers
        // are mutated.
        //
        // TODO: Find a way to do this more ergonomically and efficiently, perhaps by
        // using a safe wrapper around a custom type? We also want to make it so
        // a buffer can never be resized.
        let (left_out, right_out) = audio_through.split_first_mut().unwrap();
        let left_out = Shared::get_mut(left_out).unwrap();
        let right_out = Shared::get_mut(&mut right_out[1]).unwrap();

        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            //
            // TODO: Find a more ergonomic way to do this using a safe wrapper around a
            // custom type? We also want to make it so
            // a buffer can never be resized.
            unsafe {
                *left_out.get_unchecked_mut(i) *= gain;
                *right_out.get_unchecked_mut(i) *= gain;
            }
        }
    }
}
