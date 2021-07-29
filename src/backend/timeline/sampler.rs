use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{SampleTime, Seconds};

use crate::backend::graph_interface::{ProcInfo, StereoAudioBlockBuffer};
use crate::backend::parameter::SmoothOutput;
use crate::backend::resource_loader::{AnyPcm, MonoPcm, StereoPcm};

pub fn sample_stereo(
    proc_info: &ProcInfo,
    pcm: &StereoPcm,
    out: &mut AtomicRefMut<StereoAudioBlockBuffer>,
    pcm_start: Seconds,
    amp: &SmoothOutput<f32>,
) {
    // Very crude and temporary sampling engine. This will get better!

    let (pcm_start_smp, sub_sample) = pcm_start.to_sub_sample(proc_info.sample_rate);

    if pcm_start_smp.0 >= 0 {
        let pcm_start_smp = pcm_start_smp.0 as usize;

        if pcm_start_smp + proc_info.frames() <= pcm.len() {
            for i in 0..proc_info.frames() {
                out.left[i] = pcm.left()[pcm_start_smp + i] * amp[i];
                out.right[i] = pcm.right()[pcm_start_smp + i] * amp[i];
            }
        }
    }
}
