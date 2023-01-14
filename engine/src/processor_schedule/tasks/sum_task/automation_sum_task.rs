use smallvec::SmallVec;

use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::SharedBuffer;

pub(crate) struct AutomationSumTask {
    pub input: SmallVec<[SharedBuffer<AutomationIoEvent>; 4]>,
    pub output: SharedBuffer<AutomationIoEvent>,
}

impl AutomationSumTask {
    pub fn process(&mut self) {
        let mut out_buf = self.output.borrow_mut();
        out_buf.data.clear();

        for in_buf in self.input.iter() {
            let in_buf = in_buf.borrow();
            out_buf.data.extend_from_slice(in_buf.data.as_slice());
        }

        // TODO: Sanitize buffers with `PluginEventOutputSanitizer`?
    }
}
