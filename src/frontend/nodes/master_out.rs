use basedrop::Shared;

use crate::shared_state::{AudioGraphNode, ProcInfo};

pub struct StereoMasterOutNode {}

impl StereoMasterOutNode {}

impl AudioGraphNode for StereoMasterOutNode {
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_in_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        _proc_info: &ProcInfo,
        _audio_through: &mut Vec<Shared<Vec<f32>>>,
        _extra_audio_in: &Vec<Shared<Vec<f32>>>,
        _extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        // Master output does nothing.
    }
}
