use basedrop::Shared;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ProcInfo;
use meadowlark_plugin_api::{PluginInstanceID, ProcBuffers};
use smallvec::SmallVec;

use crate::plugin_host::event_io_buffers::PluginEventIoBuffers;
use crate::plugin_host::{PluginHostProcessorWrapper, SharedPluginHostProcessor};

pub(crate) struct PluginTask {
    pub plugin_id: PluginInstanceID,
    pub shared_processor: SharedPluginHostProcessor,
    pub current_processor: Option<Shared<PluginHostProcessorWrapper>>,

    pub buffers: ProcBuffers,
    pub event_buffers: PluginEventIoBuffers,
    pub clear_audio_in_buffers: SmallVec<[SharedBuffer<f32>; 2]>,
}

impl PluginTask {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        // Poll for a new processor if there is none.
        let current_processor =
            self.current_processor.take().unwrap_or_else(|| self.shared_processor.get());

        let drop_old_processor = if let Some(processor) = &mut *current_processor.borrow_mut() {
            processor.process(proc_info, &mut self.buffers, &mut self.event_buffers)
        } else {
            // Plugin is deactivated.
            self.buffers.bypassed(proc_info);
            self.event_buffers.bypassed();

            false
        };

        if drop_old_processor {
            // Drop the old processor if we got a request to deactivate or
            // remove the plugin.
            *current_processor.borrow_mut() = None;
        } else {
            self.current_processor = Some(current_processor);
        }
    }
}
