use atomic_refcell::{AtomicRef, AtomicRefCell};
use basedrop::Shared;

pub struct DeclickBuffers {
    pub start_stop_buf: Vec<f32>,
    pub jump_out_buf: Vec<f32>,
    pub jump_in_buf: Vec<f32>,
}

#[derive(Clone)]
pub struct DeclickInfo {
    // TODO: Explain what each of these fields mean.
    buffers: Shared<AtomicRefCell<DeclickBuffers>>,

    pub start_stop_active: bool,
    pub jump_active: bool,

    pub jump_in_playhead_frame: i64,
    pub jump_out_playhead_frame: u64,

    pub start_declick_start_frame: u64,
    pub jump_in_declick_start_frame: i64,
}

impl DeclickInfo {
    pub fn _new(
        buffers: Shared<AtomicRefCell<DeclickBuffers>>,
        start_stop_active: bool,
        jump_active: bool,
        jump_in_playhead_frame: i64,
        jump_out_playhead_frame: u64,
        start_declick_start_frame: u64,
        jump_in_declick_start_frame: i64,
    ) -> Self {
        Self {
            buffers,
            start_stop_active,
            jump_active,
            jump_in_playhead_frame,
            jump_out_playhead_frame,
            start_declick_start_frame,
            jump_in_declick_start_frame,
        }
    }

    pub fn buffers(&self) -> AtomicRef<'_, DeclickBuffers> {
        self.buffers.borrow()
    }
}
