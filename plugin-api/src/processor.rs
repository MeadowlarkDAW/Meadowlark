use std::error::Error;

use super::automation::AutomationIoEvent;
use super::buffer::EventBuffer;
use super::process_info::{ProcBuffers, ProcInfo, ProcessStatus};

/// The methods of an audio plugin instance which run in the "process" thread.
pub trait PluginProcessor: Send + 'static {
    /// This will be called when the plugin should start processing after just activing/
    /// waking up from sleep.
    ///
    /// Return an error if the plugin failed to start processing. In this case the host will not
    /// call `process()` and return the plugin to sleep.
    ///
    /// By default this just returns `Ok(())`.
    ///
    /// `[process-thread & active_state & !processing_state]`
    #[allow(unused)]
    fn start_processing(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// This will be called when the host puts the plugin to sleep.
    ///
    /// By default this trait method does nothing.
    ///
    /// `[process-thread & active_state & processing_state]`
    #[allow(unused)]
    fn stop_processing(&mut self) {}

    /// Process audio and events.
    ///
    /// `[process-thread & active_state & processing_state]`
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        in_events: &EventBuffer,
        out_events: &mut EventBuffer,
    ) -> ProcessStatus;

    /// Flushes a set of parameter changes.
    ///
    /// This will only be called while the plugin is active.
    ///
    /// This will never be called if `PluginMainThread::num_params()` returned 0.
    ///
    /// This method will not be called concurrently to clap_plugin->process().
    ///
    /// This method will not be used while the plugin is processing.
    ///
    /// By default this does nothing.
    ///
    /// [active ? process-thread : main-thread]
    #[allow(unused)]
    fn param_flush(&mut self, in_events: &EventBuffer, out_events: &mut EventBuffer) {}

    /// Only used for internal plugins which output parameter automation events for
    /// other plugins.
    ///
    /// This will only be called if `PluginMainThread::has_automation_out_port()`
    /// returned `true`.
    ///
    /// By default this returns `ProcessStatus::Error`.
    ///
    /// `[process-thread & active_state & processing_state]`
    #[allow(unused)]
    fn process_with_automation_out(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        in_events: &EventBuffer,
        out_events: &mut EventBuffer,
        automation_out: &mut Vec<AutomationIoEvent>,
    ) -> ProcessStatus {
        ProcessStatus::Error
    }
}
