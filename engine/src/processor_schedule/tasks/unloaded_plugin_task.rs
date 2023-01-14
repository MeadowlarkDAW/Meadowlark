use smallvec::SmallVec;

use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ProcInfo;

use crate::plugin_host::event_io_buffers::NoteIoEvent;

pub(crate) struct UnloadedPluginTask {
    pub audio_through: SmallVec<[(SharedBuffer<f32>, SharedBuffer<f32>); 4]>,
    pub note_through: Option<(SharedBuffer<NoteIoEvent>, SharedBuffer<NoteIoEvent>)>,

    pub clear_audio_out: SmallVec<[SharedBuffer<f32>; 4]>,
    pub clear_note_out: SmallVec<[SharedBuffer<NoteIoEvent>; 2]>,
    pub clear_automation_out: Option<SharedBuffer<AutomationIoEvent>>,
}

impl UnloadedPluginTask {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        // Pass audio through the main ports.
        for (in_buf, out_buf) in self.audio_through.iter() {
            let in_buf_ref = in_buf.borrow();
            let mut out_buf_ref = out_buf.borrow_mut();

            let in_buf_part = &in_buf_ref.data[0..proc_info.frames];
            let out_buf_part = &mut out_buf_ref.data[0..proc_info.frames];

            out_buf_part.copy_from_slice(in_buf_part);

            out_buf_ref.is_constant = in_buf_ref.is_constant;
        }

        // Pass notes through the main ports.
        if let Some((in_buf, out_buf)) = &self.note_through {
            let in_buf_ref = in_buf.borrow();
            let mut out_buf_ref = out_buf.borrow_mut();

            out_buf_ref.data.clear();
            out_buf_ref.data.clone_from(&in_buf_ref.data);
        }

        // Make sure all output buffers are cleared.
        for out_buf in self.clear_audio_out.iter() {
            out_buf.clear_and_set_constant_hint(proc_info.frames);
        }
        for out_buf in self.clear_note_out.iter() {
            out_buf.truncate();
        }
        if let Some(out_buf) = &self.clear_automation_out {
            out_buf.truncate();
        }
    }
}
