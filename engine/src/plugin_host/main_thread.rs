use audio_graph::{AudioGraphHelper, EdgeID, PortID};
use basedrop::Shared;
use clack_host::events::{Event, EventFlags, EventHeader};
use clack_host::utils::Cookie;
use meadowlark_plugin_api::event::{ParamModEvent, ParamValueEvent};
use meadowlark_plugin_api::ext::audio_ports::PluginAudioPortsExt;
use meadowlark_plugin_api::ext::gui::{EmbeddedGuiInfo, GuiResizeHints, GuiSize};
use meadowlark_plugin_api::ext::note_ports::PluginNotePortsExt;
use meadowlark_plugin_api::ext::params::{ParamID, ParamInfo, ParamInfoFlags};
use meadowlark_plugin_api::ext::timer::TimerID;
use meadowlark_plugin_api::{
    HostRequestChannelReceiver, HostRequestFlags, PluginInstanceID, PluginMainThread,
};

use fnv::{FnvHashMap, FnvHashSet};
use meadowlark_plugin_api::buffer::EventBuffer;
use raw_window_handle::RawWindowHandle;
use smallvec::SmallVec;
use std::error::Error;

use crate::engine::{timer_wheel::EngineTimerWheel, OnIdleEvent, PluginActivatedStatus};
use crate::graph::{EngineEdgeID, PortChannelID};
use crate::utils::thread_id::SharedThreadIDs;

use super::channel::{
    MainToProcParamValue, PlugHostChannelMainThread, PluginActiveState, ProcToMainParamValue,
    SharedPluginHostProcessor,
};
use super::error::{ActivatePluginError, RescanParamListError, SetParamValueError};
use super::event_io_buffers::{PluginEventOutputSanitizer, PluginIoEvent};
use super::processor::BYPASS_DECLICK_SECS;
use super::{PluginHostProcessorWrapper, PluginHostSaveState};

mod sync_ports;

struct DeactivatedEventBuffers {
    in_events: EventBuffer,
    out_events: EventBuffer,
    sanitizer: PluginEventOutputSanitizer,
}

/// The state of a parameter.
#[derive(Debug, Clone)]
pub struct ParamState {
    /// Information about this parameter.
    pub info: ParamInfo,
    /// The current value of this parameter.
    pub value: f64,
    /// The current modulation amount on this parameter.
    pub mod_amount: f64,
    /// If this is `true`, then the user is currently directly modifying
    /// this parameter.
    pub is_gesturing: bool,
}

pub struct PluginHostMainThread {
    id: PluginInstanceID,

    plug_main_thread: Box<dyn PluginMainThread>,

    port_ids: PluginHostPortIDs,
    next_port_id: u32,
    free_port_ids: Vec<PortID>,

    channel: PlugHostChannelMainThread,

    save_state: PluginHostSaveState,

    param_list: Vec<ParamID>,
    param_states: FnvHashMap<ParamID, ParamState>,
    latency: i64,
    is_loaded: bool,
    gui_active: bool,
    gui_visible: bool,
    supports_floating_gui: bool,
    supports_embedded_gui: bool,

    num_audio_in_channels: usize,
    num_audio_out_channels: usize,

    deactivated_event_buffers: Option<DeactivatedEventBuffers>,
    modified_params: Vec<ParamModifiedInfo>,

    registered_timers: FnvHashSet<TimerID>,

    host_request_rx: HostRequestChannelReceiver,
    remove_requested: bool,
    save_state_dirty: bool,
    restarting: bool,
    do_rescan_audio_ports_on_restart: bool,
    do_rescan_note_ports_on_restart: bool,
    has_activation_once: bool,
}

impl PluginHostMainThread {
    pub(crate) fn new(
        id: PluginInstanceID,
        mut save_state: PluginHostSaveState,
        mut plug_main_thread: Box<dyn PluginMainThread>,
        host_request_rx: HostRequestChannelReceiver,
        plugin_loaded: bool,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        if let Some(save_state) = save_state.raw_state.clone() {
            match plug_main_thread.load_save_state(save_state) {
                Ok(()) => {
                    log::trace!("Plugin {:?} successfully loaded save state", &id);
                }
                Err(e) => {
                    log::error!("Plugin {:?} failed to load save state: {}", &id, e);
                }
            }
        }

        // Collect the total number of audio channels to make it easier
        // for the audio graph compiler.
        let (num_audio_in_channels, num_audio_out_channels) =
            if let Some(audio_ports_ext) = &save_state.backup_audio_ports_ext {
                (audio_ports_ext.total_in_channels(), audio_ports_ext.total_out_channels())
            } else {
                (0, 0)
            };

        if save_state.backup_audio_ports_ext.is_none() {
            // Start with an empty config (no audio or note ports) if
            // the previous backup config did not exist in the save
            // state.
            save_state.backup_audio_ports_ext = Some(PluginAudioPortsExt::empty());
            save_state.backup_note_ports_ext = Some(PluginNotePortsExt::empty());
        }

        let bypassed = save_state.bypassed;

        let supports_floating_gui = plug_main_thread.supports_gui(true);
        let supports_embedded_gui = plug_main_thread.supports_gui(false);

        let mut new_self = Self {
            id: id.clone(),
            plug_main_thread,
            port_ids: PluginHostPortIDs::new(),
            next_port_id: 0,
            free_port_ids: Vec::new(),
            channel: PlugHostChannelMainThread::new(bypassed, coll_handle),
            save_state,
            param_list: Vec::new(),
            param_states: FnvHashMap::default(),
            latency: 0,
            is_loaded: plugin_loaded,
            gui_active: false,
            gui_visible: false,
            supports_floating_gui,
            supports_embedded_gui,
            num_audio_in_channels,
            num_audio_out_channels,
            deactivated_event_buffers: None,
            modified_params: Vec::new(),
            registered_timers: FnvHashSet::default(),
            host_request_rx,
            remove_requested: false,
            save_state_dirty: false,
            restarting: false,
            do_rescan_audio_ports_on_restart: false,
            do_rescan_note_ports_on_restart: false,
            has_activation_once: false,
        };

        if let Err(e) = new_self.refresh_parameter_list() {
            log::error!(
                "Failed to get parameter list after initializing plugin with ID {:?}: {}",
                id,
                e
            );
        }

        new_self
    }

    /// Returns `true` if this plugin has successfully been loaded from disk,
    /// `false` if not.
    ///
    /// This can be `false` in the case where one user opens another user's
    /// project on a different machine, and that machine does not have this
    /// plugin installed.
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    /// Returns `true` if this plugin is currently being bypassed.
    pub fn is_bypassed(&self) -> bool {
        self.save_state.bypassed
    }

    /// Bypass/unbypass this plugin.
    pub fn set_bypassed(&mut self, bypassed: bool) {
        if self.save_state.bypassed != bypassed {
            // The user has manually bypassed/unpassed this plugin, so make
            // sure it stays bypassed/unbypassed the next time it is loaded
            // from a save state.
            self.save_state.bypassed = bypassed;
            self.save_state_dirty = true;

            self.channel.shared_state.set_bypassed(bypassed);
        }
    }

    /// Tell the plugin to load the given save state.
    ///
    /// This will return `Err(e)` if the plugin failed to load the given
    /// save state.
    pub fn load_save_state(&mut self, state: Vec<u8>) -> Result<(), String> {
        self.save_state_dirty = true;
        self.plug_main_thread.load_save_state(state)
    }

    /// This will return `true` if the plugin's save state has changed
    /// since the last time its save state was collected.
    pub fn is_save_state_dirty(&self) -> bool {
        self.save_state_dirty
    }

    /// Collect the save state of this plugin.
    pub fn collect_save_state(&mut self) -> PluginHostSaveState {
        if self.save_state_dirty {
            self.save_state_dirty = false;

            let raw_state = match self.plug_main_thread.collect_save_state() {
                Ok(raw_state) => raw_state,
                Err(e) => {
                    log::error!("Failed to collect save state from plugin {:?}: {}", &self.id, e);

                    None
                }
            };

            self.save_state.raw_state = raw_state;
        }

        self.save_state.clone()
    }

    /// The list of parameters on this plugin.
    ///
    /// Use `PluginHostMainThread::param_state()` to retrieve the state of
    /// each parameter (parameter info, current value, modulation state).
    pub fn param_list(&self) -> &[ParamID] {
        &self.param_list
    }

    /// Retrieve the state of a particular parameter (parameter info,
    /// current value, modulation state).
    ///
    /// This will return `None` if the parameter with the given ID does not
    /// exist.
    pub fn param_state(&self, param_id: ParamID) -> Option<&ParamState> {
        self.param_states.get(&param_id)
    }

    /// Set the value of the given parameter.
    ///
    /// If successful, this returns the actual (clamped) value that the
    /// plugin accepted.
    pub fn set_param_value(
        &mut self,
        param_id: ParamID,
        value: f64,
    ) -> Result<f64, SetParamValueError> {
        let mut flush_on_main_thread = None;
        let res = if let Some(param_state) = self.param_states.get_mut(&param_id) {
            if param_state.info.flags.contains(ParamInfoFlags::IS_READONLY) {
                Err(SetParamValueError::ParamIsReadOnly(param_id))
            } else {
                let value = value.clamp(param_state.info.min_value, param_state.info.max_value);

                if let Some(param_queues) = &mut self.channel.param_queues {
                    param_queues.to_proc_param_value_tx.set(
                        param_id,
                        MainToProcParamValue { value, cookie: param_state.info._cookie },
                    );
                    param_queues.to_proc_param_value_tx.producer_done();
                } else {
                    flush_on_main_thread = Some((param_id, value, param_state.info._cookie));
                }

                param_state.value = value;
                self.save_state_dirty = true;

                Ok(value)
            }
        } else {
            Err(SetParamValueError::ParamDoesNotExist(param_id))
        };

        if let Some((param_id, value, cookie)) = flush_on_main_thread {
            let mut modified_params =
                self.flush_params_on_main_thread(Some((param_id, value, false, cookie)));
            self.modified_params.append(&mut modified_params);
        }

        res
    }

    /// Set the modulation amount on the given parameter.
    ///
    /// If successful, this returns the actual (clamped) modulation
    /// amount that the plugin accepted.
    pub fn set_param_mod_amount(
        &mut self,
        param_id: ParamID,
        mod_amount: f64,
    ) -> Result<f64, SetParamValueError> {
        let mut flush_on_main_thread = None;
        let res = if let Some(param_state) = self.param_states.get_mut(&param_id) {
            if param_state.info.flags.contains(ParamInfoFlags::IS_MODULATABLE) {
                Err(SetParamValueError::ParamIsNotModulatable(param_id))
            } else {
                // TODO: Clamp mod amount?

                if let Some(param_queues) = &mut self.channel.param_queues {
                    param_queues.to_proc_param_mod_tx.set(
                        param_id,
                        MainToProcParamValue {
                            value: mod_amount,
                            cookie: param_state.info._cookie,
                        },
                    );
                    param_queues.to_proc_param_mod_tx.producer_done();
                } else {
                    flush_on_main_thread = Some((param_id, mod_amount, param_state.info._cookie));
                }

                param_state.mod_amount = mod_amount;

                Ok(mod_amount)
            }
        } else {
            Err(SetParamValueError::ParamDoesNotExist(param_id))
        };

        if let Some((param_id, mod_amount, cookie)) = flush_on_main_thread {
            let mut modified_params =
                self.flush_params_on_main_thread(Some((param_id, mod_amount, true, cookie)));
            self.modified_params.append(&mut modified_params);
        }

        res
    }

    /// Get the display text for the given parameter with the given
    /// value.
    pub fn param_value_to_text(
        &self,
        param_id: ParamID,
        value: f64,
        text_buffer: &mut String,
    ) -> Result<(), String> {
        self.plug_main_thread.param_value_to_text(param_id, value, text_buffer)
    }

    /// Conver the given text input to a value for this parameter.
    pub fn param_text_to_value(&self, param_id: ParamID, text_input: &str) -> Option<f64> {
        self.plug_main_thread.param_text_to_value(param_id, text_input)
    }

    /// Returns whether or not this plugin instance supports creating a
    /// custom GUI in a floating window that the plugin manages itself.
    pub fn supports_floating_gui(&self) -> bool {
        self.supports_floating_gui
    }

    /// Returns whether or not this plugin instance supports embedding
    /// a custom GUI into a window managed by the host.
    pub fn supports_embedded_gui(&self) -> bool {
        self.supports_embedded_gui
    }

    /// Create a new floating GUI in a window managed by the plugin itself.
    pub fn create_new_floating_gui(
        &mut self,
        suggested_title: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        if self.gui_active {
            return Err(
                "Could not create new embedded GUI for plugin: plugin GUI is already active".into(),
            );
        }
        if self.remove_requested {
            return Err(
                "Ignored request to create new plugin GUI: plugin is scheduled to be removed"
                    .into(),
            );
        }
        if !self.supports_floating_gui {
            return Err(
                "Could not create new floating GUI for plugin: plugin does not support floating GUIs".into(),
            );
        }

        let res = self.plug_main_thread.create_new_floating_gui(suggested_title);

        if res.is_ok() {
            self.gui_active = true;
            self.gui_visible = false;
        }

        res
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
    pub fn create_new_embedded_gui(
        &mut self,
        scale: Option<f64>,
        size: Option<GuiSize>,
        parent_window: RawWindowHandle,
    ) -> Result<EmbeddedGuiInfo, Box<dyn Error>> {
        if self.gui_active {
            return Err(
                "Could not create new embedded GUI for plugin: plugin GUI is already active".into(),
            );
        }
        if self.remove_requested {
            return Err(
                "Ignored request to create new plugin GUI: plugin is scheduled to be removed"
                    .into(),
            );
        }
        if !self.supports_embedded_gui {
            return Err(
                "Could not create new embedded GUI for plugin: plugin does not support embedded GUIs".into(),
            );
        }

        let res = self.plug_main_thread.create_new_embedded_gui(scale, size, parent_window);

        if res.is_ok() {
            self.gui_active = true;
            self.gui_visible = false;
        }

        res
    }

    /// Destroy the currently active GUI.
    pub fn destroy_gui(&mut self) {
        if self.gui_active {
            self.plug_main_thread.destroy_gui();
            self.gui_active = false;
            self.gui_visible = false;
        } else {
            log::warn!("Ignored request to destroy plugin GUI: plugin has no active GUI");
        }
    }

    /// Information provided by the plugin to improve window resizing when initiated
    /// by the host or window manager. Only for plugins with resizable GUIs.
    pub fn gui_resize_hints(&self) -> Option<GuiResizeHints> {
        if self.gui_active {
            self.plug_main_thread.gui_resize_hints()
        } else {
            log::warn!(
                "Called `PluginHostMainThread::gui_resize_hints()` on plugin with no active GUI"
            );
            None
        }
    }

    /// If the plugin gui is resizable, then the plugin will calculate the closest
    /// usable size which fits in the given size. Only for embedded GUIs.
    ///
    /// This method does not change the size of the current GUI.
    ///
    /// If the plugin does not support changing the size of its GUI, then this
    /// will return `None`.
    pub fn adjust_gui_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        if self.gui_active {
            self.plug_main_thread.adjust_gui_size(size)
        } else {
            log::warn!(
                "Called `PluginHostMainThread::adjust_gui_size()` on plugin with no active GUI"
            );
            None
        }
    }

    /// Set the size of the plugin's GUI. Only for embedded GUIs.
    pub fn set_gui_size(&mut self, size: GuiSize) -> Result<(), Box<dyn Error>> {
        if !self.gui_active {
            return Err("Could not set GUI size for plugin: plugin has no active GUI".into());
        }

        let res = self.plug_main_thread.set_gui_size(size);

        if res.is_ok() {
            self.save_state.gui_size = Some(size);
            self.save_state_dirty = true;
        }

        res
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
    pub fn set_gui_scale(&mut self, scale: f64) -> bool {
        if self.gui_active {
            self.plug_main_thread.set_gui_scale(scale)
        } else {
            log::warn!(
                "Called `PluginHostMainThread::set_gui_scale()` on plugin with no active GUI"
            );
            false
        }
    }

    /// Show the currently active GUI.
    pub fn show_gui(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.gui_active {
            return Err("Could not show plugin GUI: plugin has no active GUI".into());
        }

        let res = self.plug_main_thread.show_gui();
        self.gui_visible = res.is_ok();
        res
    }

    /// Hide the currently active GUI.
    ///
    /// Note that hiding the GUI is not the same as destroying the GUI.
    /// Hiding only hides the window content, it does not free the GUI's
    /// resources.  Yet it may be a good idea to stop painting timers
    /// when a plugin GUI is hidden.
    pub fn hide_gui(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.gui_active {
            return Err("Could not hide plugin GUI: plugin has no active GUI".into());
        }

        let res = self.plug_main_thread.hide_gui();
        self.gui_visible = res.is_err();
        res
    }

    /// Returns `true` if the plugin currently has an actively loaded GUI.
    pub fn is_gui_active(&self) -> bool {
        self.gui_active
    }

    /// Returns `true` if the plugin currently has an actively loaded GUI
    /// that is visible.
    pub fn is_gui_visible(&self) -> bool {
        self.gui_visible
    }

    /// Returns `Ok(())` if the plugin can be activated right now.
    pub fn can_activate(&self) -> Result<(), ActivatePluginError> {
        // TODO: without this check it seems something is attempting to activate the plugin twice
        if self.channel.shared_state.get_active_state() == PluginActiveState::Active {
            return Err(ActivatePluginError::AlreadyActive);
        }
        Ok(())
    }

    /// Get the audio port configuration on this plugin.
    ///
    /// This will return `None` if this plugin is unloaded and there
    /// exists no backup of the audio ports extension.
    pub fn audio_ports_ext(&self) -> Option<&PluginAudioPortsExt> {
        self.save_state.backup_audio_ports_ext.as_ref()
    }

    /// Get the note port configuration on this plugin.
    ///
    /// This will return `None` if this plugin is unloaded and there
    /// exists no backup of the note ports extension.
    pub fn note_ports_ext(&self) -> Option<&PluginNotePortsExt> {
        self.save_state.backup_note_ports_ext.as_ref()
    }

    /// The total number of audio input channels on this plugin.
    pub fn num_audio_in_channels(&self) -> usize {
        self.num_audio_in_channels
    }

    /// The total number of audio output channels on this plugin.
    pub fn num_audio_out_channels(&self) -> usize {
        self.num_audio_out_channels
    }

    /// The latency of this plugin in frames.
    pub fn latency(&self) -> i64 {
        self.latency
    }

    /// The unique ID for this plugin instance.
    pub fn id(&self) -> &PluginInstanceID {
        &self.id
    }

    /// Schedule this plugin to be deactivated.
    ///
    /// This plugin will not be fully deactivated until the plugin host's
    /// processor is dropped in the process thread (which in turn sets the
    /// `PluginActiveState::DroppedAndReadyToDeactivate` flag).
    ///
    /// This returns the plugin host's processor, which is then sent to the
    /// new schedule to be dropped. This is necessary because otherwise it
    /// is possible that the new schedule can be sent before the old
    /// processor has a chance to drop in the process thread, causing it to
    /// be later dropped in the garbage collector thread (not what we want).
    pub(crate) fn schedule_deactivate(
        &mut self,
        coll_handle: &basedrop::Handle,
    ) -> Option<Shared<PluginHostProcessorWrapper>> {
        if self.channel.shared_state.get_active_state() != PluginActiveState::Active {
            return None;
        }

        let plug_proc_to_drop =
            Some(self.channel.drop_processor_pointer_on_main_thread(coll_handle));

        // Set a flag to alert the process thread to drop this plugin host's
        // processor.
        self.channel.shared_state.set_active_state(PluginActiveState::WaitingToDrop);

        plug_proc_to_drop
    }

    /// Schedule this plugin to be removed.
    ///
    /// This plugin will not be fully removed/dropped until the plugin host's
    /// processor is dropped in the process thread (which in turn sets the
    /// `PluginActiveState::DroppedAndReadyToDeactivate` flag).
    ///
    /// This returns the plugin host's processor, which is then sent to the
    /// new schedule to be dropped (because removing a plugin always
    /// requires the graph to recompile). This is necessary because otherwise
    /// it is possible that the new schedule can be sent before the old
    /// processor has a chance to drop in the process thread, causing it to
    /// be later dropped in the garbage collector thread (not what we want).
    pub(crate) fn schedule_remove(
        &mut self,
        coll_handle: &basedrop::Handle,
        engine_timer: &mut EngineTimerWheel,
    ) -> Option<Shared<PluginHostProcessorWrapper>> {
        self.remove_requested = true;

        if self.gui_active {
            self.plug_main_thread.destroy_gui();
        }

        engine_timer.unregister_all_timers_on_plugin(self.id.unique_id());

        self.schedule_deactivate(coll_handle)
    }

    /// Returns the plugin host's processor (wrapped in a thread-safe shared
    /// container).
    pub(crate) fn shared_processor(&self) -> &SharedPluginHostProcessor {
        self.channel.shared_processor()
    }

    /// The abstract graph's port IDs for each of the corresponding
    /// ports/channels in this plugin.
    pub(crate) fn port_ids(&self) -> &PluginHostPortIDs {
        &self.port_ids
    }

    // TODO: let the user manually activate an inactive plugin
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub(crate) fn activate(
        &mut self,
        sample_rate: u32,
        min_frames: u32,
        max_frames: u32,
        graph_helper: &mut AudioGraphHelper,
        edge_id_to_ds_edge_id: &mut FnvHashMap<EdgeID, EngineEdgeID>,
        thread_ids: SharedThreadIDs,
        schedule_version: u64,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedStatus, ActivatePluginError> {
        // Return an error if this plugin cannot be activated right now.
        self.can_activate()?;

        let set_inactive_with_error = |self_: &mut Self| {
            self_.channel.shared_state.set_active_state(PluginActiveState::InactiveWithError);
            self_.channel.param_queues = None;
        };

        // Retrieve the (new) audio ports and note ports configuration of this plugin.
        let new_audio_ports = if self.do_rescan_audio_ports_on_restart || !self.has_activation_once
        {
            self.do_rescan_audio_ports_on_restart = false;
            match self.plug_main_thread.audio_ports_ext() {
                Ok(audio_ports) => Some(audio_ports),
                Err(e) => {
                    set_inactive_with_error(self);
                    return Err(ActivatePluginError::PluginFailedToGetAudioPortsExt(e));
                }
            }
        } else {
            None
        };
        let new_note_ports = if self.do_rescan_note_ports_on_restart || !self.has_activation_once {
            self.do_rescan_note_ports_on_restart = false;
            match self.plug_main_thread.note_ports_ext() {
                Ok(note_ports) => Some(note_ports),
                Err(e) => {
                    set_inactive_with_error(self);
                    return Err(ActivatePluginError::PluginFailedToGetNotePortsExt(e));
                }
            }
        } else {
            None
        };

        self.deactivated_event_buffers = None;

        // Retrieve the (new) latency of this plugin.
        let latency = self.plug_main_thread.latency();
        let has_new_latency = if self.latency != latency {
            self.latency = latency;
            // Updates the new latency for the node in the abstract graph.
            sync_ports::sync_latency_in_graph(self, graph_helper, latency);
            true
        } else {
            false
        };

        let mut needs_recompile = has_new_latency;

        let removed_edges =
            if new_audio_ports.is_some() || new_note_ports.is_some() || !self.has_activation_once {
                // Add/remove ports from the abstract graph according to the plugin's new
                // audio ports and note ports extensions.
                //
                // On success, returns:
                // - a list of all edges that were removed as a result of the plugin
                // removing some of its ports
                // - `true` if the audio graph needs to be recompiled as a result of the
                // plugin adding/removing ports.
                let (removed_edges, recompile) = match sync_ports::sync_ports_in_graph(
                    self,
                    graph_helper,
                    edge_id_to_ds_edge_id,
                    &new_audio_ports,
                    &new_note_ports,
                    coll_handle,
                ) {
                    Ok((removed_edges, needs_recompile)) => (removed_edges, needs_recompile),
                    Err(e) => {
                        set_inactive_with_error(self);
                        return Err(e);
                    }
                };

                needs_recompile |= recompile;
                removed_edges
            } else {
                Vec::new()
            };

        // Attempt to activate the plugin.
        match self.plug_main_thread.activate(sample_rate, min_frames, max_frames, coll_handle) {
            Ok(info) => {
                let audio_ports_changed = if let Some(new_audio_ports) = &new_audio_ports {
                    if let Some(old_audio_ports) = &self.save_state.backup_audio_ports_ext {
                        old_audio_ports != new_audio_ports
                    } else {
                        true
                    }
                } else {
                    false
                };

                let note_ports_changed = if let Some(new_note_ports) = &new_note_ports {
                    if let Some(old_note_ports) = &self.save_state.backup_note_ports_ext {
                        old_note_ports != new_note_ports
                    } else {
                        true
                    }
                } else {
                    false
                };

                let has_new_audio_ports_ext = if audio_ports_changed {
                    let new_audio_ports = new_audio_ports.unwrap();

                    self.num_audio_in_channels = new_audio_ports.total_in_channels();
                    self.num_audio_out_channels = new_audio_ports.total_out_channels();

                    self.save_state.backup_audio_ports_ext = Some(new_audio_ports);
                    true
                } else {
                    false
                };
                let has_new_note_ports_ext = if note_ports_changed {
                    let new_note_ports = new_note_ports.unwrap();
                    self.save_state.backup_note_ports_ext = Some(new_note_ports);
                    true
                } else {
                    false
                };

                // If the plugin restarting requires the graph to recompile first (because
                // the port configuration or latency configuration has changed), tell the
                // new processor to wait for the new schedule before processing.
                let sched_version =
                    if needs_recompile { schedule_version + 1 } else { schedule_version };

                // The number of frames to smooth/declick the audio outputs when
                // bypassing/unbypassing the plugin.
                let bypass_declick_frames =
                    (BYPASS_DECLICK_SECS * f64::from(sample_rate)).round() as usize;

                // Send the new processor to the process thread.
                self.channel.shared_state.set_active_state(PluginActiveState::Active);
                self.channel.new_processor(
                    info.processor,
                    self.id.unique_id(),
                    self.param_list.len(),
                    thread_ids,
                    sched_version,
                    bypass_declick_frames,
                    coll_handle,
                );

                // Make sure that the new configurations are saved in the save state of
                // this plugin.
                self.save_state.active = true;
                self.save_state_dirty = true;
                self.has_activation_once = true;

                Ok(PluginActivatedStatus {
                    has_new_audio_ports_ext,
                    has_new_note_ports_ext,
                    internal_handle: info.internal_handle,
                    has_new_latency,
                    removed_edges,
                    caused_recompile: needs_recompile,
                })
            }
            Err(e) => {
                set_inactive_with_error(self);
                Err(ActivatePluginError::PluginSpecific(e))
            }
        }
    }

    pub(crate) fn on_timer(&mut self, timer_id: TimerID, engine_timer: &mut EngineTimerWheel) {
        // Make sure that plugin hasn't requested to unregister the timer
        // before calling its `on_timer()` method.
        if self.host_request_rx.has_timer_request() {
            let timer_requests = self.host_request_rx.fetch_timer_requests();
            for req in timer_requests.iter() {
                if req.register {
                    engine_timer.register_plugin_timer(
                        self.id.unique_id(),
                        req.timer_id,
                        req.period_ms,
                    );

                    self.registered_timers.insert(req.timer_id);
                } else {
                    engine_timer.unregister_plugin_timer(self.id.unique_id(), req.timer_id);

                    self.registered_timers.remove(&req.timer_id);
                }
            }
        }

        if self.registered_timers.contains(&timer_id) {
            self.plug_main_thread.on_timer(timer_id);
        }
    }

    fn refresh_parameter_list(&mut self) -> Result<(), RescanParamListError> {
        let error_clear = |self_: &mut Self| {
            self_.channel.param_queues = None;
            self_.param_list.clear();
            self_.param_states.clear();
        };

        let old_param_states = self.param_states.clone();
        self.param_list.clear();
        self.param_states.clear();
        self.deactivated_event_buffers = None;

        let num_params = self.plug_main_thread.num_params() as usize;
        self.param_list.reserve(num_params);
        self.param_states.reserve(num_params);

        for i in 0..num_params {
            match self.plug_main_thread.param_info(i) {
                Ok(info) => match self.plug_main_thread.param_value(info.stable_id) {
                    Ok(value) => {
                        let id = info.stable_id;
                        let (is_gesturing, mod_amount) = old_param_states
                            .get(&info.stable_id)
                            .map(|i| (i.is_gesturing, i.mod_amount))
                            .unwrap_or((false, 0.0));
                        let param_state = ParamState { info, value, is_gesturing, mod_amount };

                        if self.param_states.insert(id, param_state).is_some() {
                            error_clear(self);
                            return Err(RescanParamListError::DuplicateParamID(id));
                        }

                        self.param_list.push(id);
                    }
                    Err(e) => {
                        error_clear(self);
                        return Err(RescanParamListError::FailedToGetParamValue {
                            id: info.stable_id,
                            error_msg: format!("{}", e),
                        });
                    }
                },
                Err(e) => {
                    error_clear(self);
                    return Err(RescanParamListError::FailedToGetParamInfo {
                        index: i,
                        error_msg: format!("{}", e),
                    });
                }
            }
        }

        Ok(())
    }

    fn flush_params_on_main_thread(
        &mut self,
        in_param_event: Option<(ParamID, f64, bool, Cookie)>,
    ) -> Vec<ParamModifiedInfo> {
        let mut modified_params: Vec<ParamModifiedInfo> = Vec::new();

        if !self.channel.shared_state.get_active_state().is_active() && !self.param_list.is_empty()
        {
            if self.deactivated_event_buffers.is_none() {
                self.deactivated_event_buffers = Some(DeactivatedEventBuffers {
                    in_events: EventBuffer::with_capacity(4),
                    out_events: EventBuffer::with_capacity(self.param_list.len() * 3),
                    sanitizer: PluginEventOutputSanitizer::new(self.param_list.len()),
                });
            }

            let deactivated_event_buffers = self.deactivated_event_buffers.as_mut().unwrap();
            deactivated_event_buffers.in_events.clear();
            deactivated_event_buffers.out_events.clear();

            if let Some((param_id, value, is_mod, cookie)) = in_param_event {
                if is_mod {
                    let event = ParamModEvent::new(
                        EventHeader::new_core(0, EventFlags::empty()),
                        cookie,
                        -1, // note_id
                        param_id.as_u32(),
                        -1, // port_index
                        -1, // channel
                        -1, // key
                        value,
                    );

                    deactivated_event_buffers.in_events.push(event.as_unknown());
                } else {
                    let event = ParamValueEvent::new(
                        EventHeader::new_core(0, EventFlags::empty()),
                        cookie,
                        -1, // note_id
                        param_id.as_u32(),
                        -1, // port_index
                        -1, // channel
                        -1, // key
                        value,
                    );

                    deactivated_event_buffers.in_events.push(event.as_unknown());
                }
            }

            self.plug_main_thread.param_flush(
                &deactivated_event_buffers.in_events,
                &mut deactivated_event_buffers.out_events,
            );

            let events_iter = deactivated_event_buffers
                .out_events
                .iter()
                .filter_map(PluginIoEvent::read_from_clap);
            let events_iter = deactivated_event_buffers.sanitizer.sanitize(events_iter, None);

            for event in events_iter {
                if let PluginIoEvent::AutomationEvent { event } = event {
                    if let Some(new_value) =
                        ProcToMainParamValue::from_param_event(event.event_type)
                    {
                        let param_id = ParamID::new(event.parameter_id);
                        if let Some(param_state) = self.param_states.get_mut(&param_id) {
                            if let Some(gesture) = new_value.gesture {
                                param_state.is_gesturing = gesture.is_begin;

                                if !gesture.is_begin {
                                    // Only mark the state dirty once the user has finished adjusting
                                    // the parameter.
                                    self.save_state_dirty = true;
                                }
                            } else {
                                self.save_state_dirty = true;
                            };

                            if let Some(v) = new_value.value {
                                param_state.value = v;
                            }

                            modified_params.push(ParamModifiedInfo {
                                param_id,
                                new_value: new_value.value,
                                is_gesturing: param_state.is_gesturing,
                            })
                        }
                    }
                }
            }
        }

        modified_params
    }

    /// Poll parameter updates and requests from the plugin and the plugin host's
    /// processor.
    ///
    /// Returns the status of this plugin, along with a list of any parameters
    /// that were modified inside the plugin's custom GUI.
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub(crate) fn on_idle(
        &mut self,
        sample_rate: u32,
        min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
        graph_helper: &mut AudioGraphHelper,
        events_out: &mut SmallVec<[OnIdleEvent; 32]>,
        edge_id_to_ds_edge_id: &mut FnvHashMap<EdgeID, EngineEdgeID>,
        thread_ids: &SharedThreadIDs,
        schedule_version: u64,
        engine_timer: &mut EngineTimerWheel,
    ) -> (OnIdleResult, SmallVec<[ParamModifiedInfo; 4]>, Option<Shared<PluginHostProcessorWrapper>>)
    {
        let mut modified_params: SmallVec<[ParamModifiedInfo; 4]> = SmallVec::new();
        let mut processor_to_drop = None;

        // Get the latest request flags and activation state.
        let request_flags = self.host_request_rx.fetch_requests();
        let mut active_state = self.channel.shared_state.get_active_state();

        // Collect any parameter updates that happened in previous calls to
        // `flush_params_on_main_thread()`.
        for event in self.modified_params.drain(..) {
            modified_params.push(event);
        }

        // Poll for parameter updates from the plugin host's processor.
        if let Some(params_queue) = &mut self.channel.param_queues {
            params_queue.from_proc_param_value_rx.consume(|param_id, new_value| {
                if let Some(param_state) = &mut self.param_states.get_mut(param_id) {
                    if let Some(gesture) = new_value.gesture {
                        param_state.is_gesturing = gesture.is_begin;

                        if !gesture.is_begin {
                            // Only mark the state dirty once the user has finished adjusting
                            // the parameter.
                            self.save_state_dirty = true;
                        }
                    } else {
                        self.save_state_dirty = true;
                    };

                    if let Some(v) = new_value.value {
                        param_state.value = v;
                    }

                    modified_params.push(ParamModifiedInfo {
                        param_id: *param_id,
                        new_value: new_value.value,
                        is_gesturing: param_state.is_gesturing,
                    });
                }
            });
        }

        // More often than not, these flags will be empty. So optimize by only checking
        // individual flags when necessary.
        if !request_flags.is_empty() {
            if request_flags.contains(HostRequestFlags::MARK_DIRTY) {
                log::trace!("Plugin {:?} manually marked its state as dirty", &self.id);

                // The plugin has manually changed its save state, so mark the state
                // as dirty so it can be collected later.
                self.save_state_dirty = true;
            }

            if request_flags.contains(HostRequestFlags::CALLBACK) {
                log::trace!("Plugin {:?} requested the host call `on_main_thread()`", &self.id);

                self.plug_main_thread.on_main_thread();
            }

            if request_flags.contains(HostRequestFlags::RESCAN_PARAMS) {
                log::debug!("Plugin {:?} requested the host rescan its parameters", &self.id);

                let res = self.refresh_parameter_list();

                if let Err(e) = &res {
                    log::error!(
                        "Failed to get parameter list on plugin with ID {:?} after request to rescan parameters: {}",
                        &self.id,
                        e
                    );
                }

                events_out.push(OnIdleEvent::PluginUpdatedParameterList {
                    plugin_id: self.id.clone(),
                    status: res,
                });
            }

            if request_flags.contains(HostRequestFlags::FLUSH_PARAMS) {
                log::trace!("Plugin {:?} requested the host flush its parameters", &self.id);

                if active_state.is_active() {
                    self.channel.shared_state.set_param_flush_requested();
                } else {
                    self.modified_params = self.flush_params_on_main_thread(None);
                    for event in self.modified_params.drain(..) {
                        modified_params.push(event);
                    }
                }
            }

            if request_flags.intersects(
                HostRequestFlags::RESTART
                    | HostRequestFlags::RESCAN_AUDIO_PORTS
                    | HostRequestFlags::RESCAN_NOTE_PORTS,
            ) {
                // The plugin has requested the host to restart the plugin (or rescan its
                // audio and/or note ports).
                //
                // We just do a full restart and rescan for all "rescan port" requests for
                // simplicity. I don't expect plugins to change the state of their ports
                // often anyway.

                if request_flags.contains(HostRequestFlags::RESTART) {
                    log::debug!("Plugin {:?} requested the host to restart the plugin", &self.id);
                }

                let rescan_audio_ports =
                    request_flags.contains(HostRequestFlags::RESCAN_AUDIO_PORTS);
                let rescan_note_ports = request_flags.contains(HostRequestFlags::RESCAN_NOTE_PORTS);

                if rescan_audio_ports {
                    self.do_rescan_audio_ports_on_restart = true;
                    log::debug!(
                        "Plugin {:?} requested the host to rescan its audio ports",
                        &self.id
                    );
                }
                if rescan_note_ports {
                    self.do_rescan_note_ports_on_restart = true;
                    log::debug!(
                        "Plugin {:?} requested the host to rescan its note ports",
                        &self.id
                    );
                }

                self.restarting = true;

                if active_state == PluginActiveState::Active {
                    processor_to_drop = self.schedule_deactivate(coll_handle);
                    active_state = PluginActiveState::WaitingToDrop;
                }
            }

            if request_flags
                .intersects(HostRequestFlags::GUI_CLOSED | HostRequestFlags::GUI_DESTROYED)
            {
                log::trace!("Plugin {:?} has closed its custom GUI", &self.id);

                let was_destroyed = request_flags.contains(HostRequestFlags::GUI_DESTROYED);
                if was_destroyed {
                    // As per the spec, we must call `destroy()` to acknowledge the GUI
                    // destruction.
                    if self.gui_active {
                        self.plug_main_thread.destroy_gui();
                    }
                }

                events_out.push(OnIdleEvent::PluginGuiClosed {
                    plugin_id: self.id.clone(),
                    was_destroyed,
                });
            }

            if request_flags.contains(HostRequestFlags::GUI_SHOW) {
                log::trace!("Plugin {:?} requested the host to show its GUI", &self.id);

                events_out
                    .push(OnIdleEvent::PluginRequestedToShowGui { plugin_id: self.id.clone() });
            }

            if request_flags.contains(HostRequestFlags::GUI_HIDE) {
                log::trace!("Plugin {:?} requested the host to hide its GUI", &self.id);

                events_out
                    .push(OnIdleEvent::PluginRequestedToHideGui { plugin_id: self.id.clone() });
            }

            if request_flags.contains(HostRequestFlags::GUI_RESIZE) {
                log::trace!("Plugin {:?} requested the host to resize its GUI", &self.id);

                if let Some(size) = self.host_request_rx.fetch_gui_size_request() {
                    events_out.push(OnIdleEvent::PluginRequestedToResizeGui {
                        plugin_id: self.id.clone(),
                        size,
                    });
                }
            }

            if request_flags.contains(HostRequestFlags::GUI_HINTS_CHANGED) {
                let resize_hints = self.plug_main_thread.gui_resize_hints();

                log::trace!(
                    "Plugin {:?} has changed its gui resize hints to {:?}",
                    &self.id,
                    &resize_hints
                );

                events_out.push(OnIdleEvent::PluginChangedGuiResizeHints {
                    plugin_id: self.id.clone(),
                    resize_hints,
                });
            }

            if request_flags.contains(HostRequestFlags::TIMER_REQUEST) {
                let timer_requests = self.host_request_rx.fetch_timer_requests();
                for req in timer_requests.iter() {
                    if req.register {
                        engine_timer.register_plugin_timer(
                            self.id.unique_id(),
                            req.timer_id,
                            req.period_ms,
                        );

                        self.registered_timers.insert(req.timer_id);
                    } else {
                        engine_timer.unregister_plugin_timer(self.id.unique_id(), req.timer_id);

                        self.registered_timers.remove(&req.timer_id);
                    }
                }
            }

            if request_flags.contains(HostRequestFlags::PROCESS)
                && !self.remove_requested
                && !self.restarting
                && active_state != PluginActiveState::DroppedAndReadyToDeactivate
            {
                log::trace!(
                    "Plugin {:?} requested the host to start processing the plugin",
                    &self.id
                );

                // The plugin has requested the host to start processing this plugin.

                if active_state == PluginActiveState::Active {
                    self.channel.shared_state.set_process_requested();
                } else if active_state == PluginActiveState::Inactive
                    || active_state == PluginActiveState::InactiveWithError
                {
                    let res = match self.activate(
                        sample_rate,
                        min_frames,
                        max_frames,
                        graph_helper,
                        edge_id_to_ds_edge_id,
                        thread_ids.clone(),
                        schedule_version,
                        coll_handle,
                    ) {
                        Ok(r) => {
                            self.save_state_dirty = true;

                            OnIdleResult::PluginActivated(r)
                        }
                        Err(e) => OnIdleResult::PluginFailedToActivate(e),
                    };

                    return (res, modified_params, processor_to_drop);
                }
            }
        }

        if active_state == PluginActiveState::DroppedAndReadyToDeactivate {
            // The plugin host's processor has successfully been dropped after
            // scheduling this plugin to be deactivated, so it is safe to fully
            // deactivate this plugin now.

            self.plug_main_thread.deactivate();
            self.channel.shared_state.set_active_state(PluginActiveState::Inactive);

            if !self.remove_requested {
                let mut res = OnIdleResult::PluginDeactivated;

                if self.restarting || request_flags.contains(HostRequestFlags::PROCESS) {
                    // The plugin has requested to be reactivated after being deactivated.

                    match self.activate(
                        sample_rate,
                        min_frames,
                        max_frames,
                        graph_helper,
                        edge_id_to_ds_edge_id,
                        thread_ids.clone(),
                        schedule_version,
                        coll_handle,
                    ) {
                        Ok(r) => {
                            self.save_state_dirty = true;
                            res = OnIdleResult::PluginActivated(r);
                        }
                        Err(e) => res = OnIdleResult::PluginFailedToActivate(e),
                    }
                } else {
                    // The user has manually deactivated this plugin, so make sure
                    // it stays deactivated the next time it is loaded from a save
                    // state.
                    self.save_state.active = false;
                    self.save_state_dirty = true;
                }

                return (res, modified_params, processor_to_drop);
            } else {
                // Plugin is ready to be fully removed/dropped.
                return (OnIdleResult::PluginReadyToRemove, modified_params, processor_to_drop);
            }
        }

        (OnIdleResult::Ok, modified_params, processor_to_drop)
    }
}

/// The abstract graph's port IDs for each of the corresponding
/// ports/channels in this plugin.
pub(crate) struct PluginHostPortIDs {
    /// Maps the engine's id for this port/channel to the abstract graph's
    /// port ID.
    pub channel_id_to_port_id: FnvHashMap<PortChannelID, PortID>,
    /// Maps the abstract graph's port ID to the engine's id for this
    /// port/channel.
    pub port_id_to_channel_id: FnvHashMap<PortID, PortChannelID>,

    /// The abstract graph's port IDs for each channel in the main audio
    /// input port.
    pub main_audio_in_port_ids: Vec<PortID>,
    /// The abstract graph's port IDs for each channel in the main audio
    /// output port.
    pub main_audio_out_port_ids: Vec<PortID>,

    /// The abstract graph's port ID for the main note input port.
    pub main_note_in_port_id: Option<PortID>,
    /// The abstract graph's port ID for the main note output port.
    pub main_note_out_port_id: Option<PortID>,
    /// The abstract graph's port ID for the main automation input port.
    pub automation_in_port_id: Option<PortID>,
    /// The abstract graph's port ID for the main automation output port.
    pub automation_out_port_id: Option<PortID>,
}

impl PluginHostPortIDs {
    pub fn new() -> Self {
        Self {
            channel_id_to_port_id: FnvHashMap::default(),
            port_id_to_channel_id: FnvHashMap::default(),
            main_audio_in_port_ids: Vec::new(),
            main_audio_out_port_ids: Vec::new(),
            main_note_in_port_id: None,
            main_note_out_port_id: None,
            automation_in_port_id: None,
            automation_out_port_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamModifiedInfo {
    pub param_id: ParamID,
    pub new_value: Option<f64>,
    pub is_gesturing: bool,
}

pub(crate) enum OnIdleResult {
    Ok,
    PluginDeactivated,
    PluginActivated(PluginActivatedStatus),
    PluginReadyToRemove,
    PluginFailedToActivate(ActivatePluginError),
}
