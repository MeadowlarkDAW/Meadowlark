use meadowlark_plugin_api::buffer::SharedBuffer;
use smallvec::SmallVec;

#[derive(Default)]
pub(crate) struct GraphInTask {
    pub audio_in: SmallVec<[SharedBuffer<f32>; 8]>,
}

#[derive(Default)]
pub(crate) struct GraphOutTask {
    pub audio_out: SmallVec<[SharedBuffer<f32>; 8]>,
}
