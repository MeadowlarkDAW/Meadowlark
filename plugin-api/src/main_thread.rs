use clack_extensions::gui::{GuiResizeHints, GuiSize};
use clack_host::events::io::EventBuffer;
use raw_window_handle::RawWindowHandle;
use std::error::Error;

use crate::ext::timer::TimerID;

use super::{ext, ParamID, PluginProcessor};

/// The methods of an audio plugin instance which run in the "main" thread.
pub trait PluginMainThread {
    /// Activate the plugin, and return the `PluginProcessor` counterpart.
    ///
    /// In this call the plugin may allocate memory and prepare everything needed for the process
    /// call. The process's sample rate will be constant and process's frame count will included in
    /// the `[min, max]` range, which is bounded by `[1, INT32_MAX]`.
    ///
    /// A `basedrop` collector handle is provided for realtime-safe garbage collection.
    ///
    /// Once activated the latency and port configuration must remain constant, until deactivation.
    ///
    /// `[main-thread & !active_state]`
    fn activate(
        &mut self,
        sample_rate: u32,
        min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String>;

    /// Collect the save state/preset of this plugin as raw bytes (use serde and bincode).
    ///
    /// If `Ok(None)` is returned, then it means that the plugin does not have a
    /// state it needs to save.
    ///
    /// By default this returns `None`.
    ///
    /// `[main-thread]`
    fn collect_save_state(&mut self) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }

    /// Load the given save state/preset (use serde and bincode).
    ///
    /// By default this does nothing.
    ///
    /// `[main-thread]`
    #[allow(unused)]
    fn load_save_state(&mut self, state: Vec<u8>) -> Result<(), String> {
        Ok(())
    }

    /// Deactivate the plugin. When this is called it also means that the `PluginProcessor`
    /// counterpart will already have been dropped.
    ///
    /// `[main-thread & active_state]`
    fn deactivate(&mut self) {}

    /// Called by the host on the main thread in response to a previous call to `host.request_callback()`.
    ///
    /// By default this does nothing.
    ///
    /// [main-thread]
    #[allow(unused)]
    fn on_main_thread(&mut self) {}

    /// An optional extension that describes the configuration of audio ports on this plugin instance.
    ///
    /// This will only be called while the plugin is inactive.
    ///
    /// The default configuration is one with no audio ports.
    ///
    /// [main-thread & !active_state]
    #[allow(unused)]
    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::EMPTY_AUDIO_PORTS_CONFIG.clone())
    }

    /// An optional extension that describes the configuration of note ports on this plugin instance.
    ///
    /// This will only be called while the plugin is inactive.
    ///
    /// The default configuration is one with no note ports.
    ///
    /// [main-thread & !active_state]
    #[allow(unused)]
    fn note_ports_ext(&mut self) -> Result<ext::note_ports::PluginNotePortsExt, String> {
        Ok(ext::note_ports::EMPTY_NOTE_PORTS_CONFIG.clone())
    }

    /// The latency in frames this plugin adds.
    ///
    /// The plugin is only allowed to change its latency when it is deactivated.
    ///
    /// By default this returns `0` (no latency).
    ///
    /// [main-thread & !active_state]
    fn latency(&self) -> i64 {
        0
    }

    // --- Parameters ---------------------------------------------------------------------------------

    /// Get the total number of parameters in this plugin.
    ///
    /// You may return 0 if this plugins has no parameters.
    ///
    /// By default this returns 0.
    ///
    /// [main-thread]
    #[allow(unused)]
    fn num_params(&mut self) -> u32 {
        0
    }

    /// Get the info of the given parameter.
    ///
    /// (Note this is takes the index of the parameter as input (length given by `num_params()`), *NOT* the ID of the parameter)
    ///
    /// This will never be called if `PluginMainThread::num_params()` returned 0.
    ///
    /// By default this returns an error.
    ///
    /// [main-thread]
    #[allow(unused)]
    fn param_info(&mut self, param_index: usize) -> Result<ext::params::ParamInfo, Box<dyn Error>> {
        Err(format!("Param at index {} does not exist", param_index).into())
    }

    /// Get the plain value of the parameter.
    ///
    /// This will never be called if `PluginMainThread::num_params()` returned 0.
    ///
    /// By default this returns an error.
    ///
    /// [main-thread]
    #[allow(unused)]
    fn param_value(&self, param_id: ParamID) -> Result<f64, Box<dyn Error>> {
        Err(format!("Param with id {:?} does not exist", param_id).into())
    }

    /// Format the display text for the given parameter value.
    ///
    /// This will never be called if `PluginMainThread::num_params()` returned 0.
    ///
    /// By default this returns `Err(())`
    ///
    /// [main-thread]
    #[allow(unused)]
    fn param_value_to_text(
        &self,
        param_id: ParamID,
        value: f64,
        text_buffer: &mut String,
    ) -> Result<(), String> {
        Err(String::new())
    }

    /// Convert the text input to a parameter value.
    ///
    /// This will never be called if `PluginMainThread::num_params()` returned 0.
    ///
    /// By default this returns `None`
    ///
    /// [main-thread]
    #[allow(unused)]
    fn param_text_to_value(&self, param_id: ParamID, text_input: &str) -> Option<f64> {
        None
    }

    /// Whether or not this plugin has an automation out port (seperate from audio and note
    /// out ports).
    ///
    /// Only return `true` for internal plugins which output parameter automation events for
    /// other plugins.
    ///
    /// By default this returns `false`.
    ///
    /// [main-thread]
    fn has_automation_out_port(&self) -> bool {
        false
    }

    /// Flushes a set of parameter changes.
    ///
    /// This will only be called while the plugin is inactive.
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

    // --- GUI ---------------------------------------------------------------------------------

    /// If `floating` is `true`, then this returns whether or not this plugin instance supports
    /// creating a custom GUI in a floating window that the plugin manages itself.
    ///
    /// If `floating` is `false`, then this returns whether or not this plugin instance supports
    /// embedding a custom GUI into a window managed by the host.
    ///
    /// By default this returns `false` for both cases.
    #[allow(unused)]
    fn supports_gui(&self, floating: bool) -> bool {
        false
    }

    /// Create a new floating GUI in a window managed by the plugin itself.
    ///
    /// By default this returns an error.
    #[allow(unused)]
    fn create_new_floating_gui(
        &mut self,
        suggested_title: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        Err("Plugin does not support a custom floating GUI".into())
    }

    /// Create a new embedded GUI in a window managed by the host.
    ///
    /// * `scale` - The absolute GUI scaling factor. This overrides any OS info.
    ///     * This should not be used if the windowing API relies upon logical pixels
    /// (such as Cocoa on MacOS).
    ///     * If this plugin prefers to work out the scaling factor itself by querying
    /// the OS directly, then ignore this value.
    /// * `size`
    ///     * If the plugin's GUI is resizable, and the size is known from previous a
    /// previous session, then put the size from that previous session here.
    ///     * If the plugin's GUI is not resizable, then this will be ignored.
    /// * `parent_window` - The `RawWindowHandle` of the window that the GUI should be
    /// embedded into.
    ///
    /// By default this returns an error.
    #[allow(unused)]
    fn create_new_embedded_gui(
        &mut self,
        scale: Option<f64>,
        size: Option<ext::gui::GuiSize>,
        parent_window: RawWindowHandle,
    ) -> Result<ext::gui::EmbeddedGuiInfo, Box<dyn Error>> {
        Err("Plugin does not support a custom embedded GUI".into())
    }

    /// Destroy the currently active GUI.
    ///
    /// By default this does nothing.
    fn destroy_gui(&mut self) {}

    /// Information provided by the plugin to improve window resizing when initiated
    /// by the host or window manager. Only for plugins with resizable GUIs.
    ///
    /// By default this returns `None`.
    fn gui_resize_hints(&self) -> Option<GuiResizeHints> {
        None
    }

    /// If the plugin gui is resizable, then the plugin will calculate the closest
    /// usable size which fits in the given size. Only for embedded GUIs.
    ///
    /// This method does not change the size of the current GUI.
    ///
    /// If the plugin does not support changing the size of its GUI, then this
    /// will return `None`.
    ///
    /// By default this returns `None`.
    #[allow(unused)]
    fn adjust_gui_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        None
    }

    /// Set the size of the plugin's GUI. Only for embedded GUIs.
    #[allow(unused)]
    fn set_gui_size(&mut self, size: GuiSize) -> Result<(), Box<dyn Error>> {
        Err("Plugin does not support a custom embedded GUI".into())
    }

    /// Set the absolute GUI scaling factor. This overrides any OS info.
    ///
    /// This should not be used if the windowing API relies upon logical pixels
    /// (such as Cocoa on MacOS).
    ///
    /// If this plugin prefers to work out the scaling factor itself by querying
    /// the OS directly, then ignore this value.
    ///
    /// Returns `true` if the plugin applied the scaling.
    /// Returns `false` if the plugin could not apply the scaling, or if the
    /// plugin ignored the request.
    ///
    /// By default this returns `false`.
    #[allow(unused)]
    fn set_gui_scale(&mut self, scale: f64) -> bool {
        false
    }

    /// Show the currently active GUI.
    ///
    /// By default this returns an error.
    fn show_gui(&mut self) -> Result<(), Box<dyn Error>> {
        Err("Plugin does not support a custom GUI".into())
    }

    /// Hide the currently active GUI.
    ///
    /// Note that hiding the GUI is not the same as destroying the GUI.
    /// Hiding only hides the window content, it does not free the GUI's
    /// resources.  Yet it may be a good idea to stop painting timers
    /// when a plugin GUI is hidden.
    ///
    /// By default this returns an error.
    fn hide_gui(&mut self) -> Result<(), Box<dyn Error>> {
        Err("Plugin does not support a custom GUI".into())
    }

    // --- Timer -------------------------------------------------------------------------------

    #[allow(unused)]
    fn on_timer(&mut self, timer_id: TimerID) {}
}

pub struct PluginActivatedInfo {
    pub processor: Box<dyn PluginProcessor>,
    pub internal_handle: Option<Box<dyn std::any::Any + Send + 'static>>,
}
