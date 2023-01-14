use atomic_refcell::{AtomicRef, AtomicRefMut};
use clack_extensions::audio_ports::{AudioPortFlags, AudioPortInfoBuffer, PluginAudioPorts};
use clack_extensions::gui::GuiApiType;
use clack_extensions::note_ports::{NotePortInfoBuffer, PluginNotePorts};
use clack_extensions::timer::TimerId;
use clack_host::events::io::{InputEvents, OutputEvents};
use clack_host::instance::processor::PluginAudioProcessor;
use clack_host::instance::{PluginAudioConfiguration, PluginInstance};
use meadowlark_plugin_api::buffer::{BufferInner, RawAudioChannelBuffers};
use meadowlark_plugin_api::ext::audio_ports::{
    AudioPortInfo, MainPortsLayout, PluginAudioPortsExt,
};
use meadowlark_plugin_api::ext::gui::{EmbeddedGuiInfo, GuiSize};
use meadowlark_plugin_api::ext::note_ports::{NotePortInfo, PluginNotePortsExt};
use meadowlark_plugin_api::ext::params::{ParamID, ParamInfo, ParamInfoFlags};
use meadowlark_plugin_api::{
    buffer::EventBuffer, ext, PluginActivatedInfo, PluginMainThread, PluginProcessor, ProcBuffers,
    ProcInfo, ProcessStatus,
};
use raw_window_handle::RawWindowHandle;
use smallvec::SmallVec;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::mem::MaybeUninit;

use super::process::ClapProcess;
use super::*;

#[derive(Default)]
pub(super) struct AudioPortChannels {
    pub num_input_ports: usize,
    pub num_output_ports: usize,
    pub max_input_channels: usize,
    pub max_output_channels: usize,
}

impl ClapPluginMainThread {
    pub(crate) fn new(instance: PluginInstance<ClapHost>) -> Result<Self, String> {
        Ok(Self { instance, audio_port_channels: AudioPortChannels::default() })
    }

    #[inline]
    fn id(&self) -> &str {
        &self.instance.shared_host_data().id
    }

    fn parse_audio_ports_extension(
        instance: &PluginInstance<ClapHost>,
    ) -> Result<PluginAudioPortsExt, String> {
        let id = &*instance.shared_host_data().id;
        log::trace!("clap plugin instance parse audio ports extension {}", id);

        if instance.is_active() {
            return Err("Cannot get audio ports extension while plugin is active".into());
        }

        let audio_ports = match instance.shared_plugin_data().get_extension::<PluginAudioPorts>() {
            None => return Ok(PluginAudioPortsExt::empty()),
            Some(e) => e,
        };

        let plugin = instance.main_thread_plugin_data();

        let num_in_ports = audio_ports.count(&plugin, true);
        let num_out_ports = audio_ports.count(&plugin, false);

        let mut buffer = AudioPortInfoBuffer::new();

        let mut has_main_in_port = false;
        let mut has_main_out_port = false;

        let inputs: Vec<AudioPortInfo> = (0..num_in_ports).filter_map(|i| {
            let raw_info = match audio_ports.get(&plugin, i, true, &mut buffer) {
                None => {
                    log::warn!("Error when getting CLAP Port Info from plugin instance {}: plugin returned no info for index {}", id, i);
                    return None;
                },
                Some(i) => i
            };

            let port_type = raw_info.port_type.and_then(|t| Some(t.0.to_str().ok()?.to_string()));

            let display_name = match raw_info.name.to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    log::warn!("Failed to get clap_audio_port_info.name from plugin instance {}", id);
                    None
                }
            };

            if raw_info.flags.contains(AudioPortFlags::IS_MAIN) {
                if has_main_in_port {
                    log::warn!("Plugin instance {} already has a main input port (at port index {})", id, i)
                } else {
                    has_main_in_port = true;
                }
            }

            Some(AudioPortInfo {
                stable_id: raw_info.id,
                channels: raw_info.channel_count as u16,
                port_type,
                display_name,
            })
        }).collect();

        let outputs: Vec<AudioPortInfo> = (0..num_out_ports).filter_map(|i| {
            let raw_info = match audio_ports.get(&plugin, i, false, &mut buffer) {
                None => {
                    log::warn!("Error when getting CLAP audio port info from plugin instance {}: plugin returned no info for index {}", id, i);
                    return None;
                },
                Some(i) => i
            };

            let port_type = raw_info.port_type.and_then(|t| Some(t.0.to_str().ok()?.to_string()));

            let display_name = match raw_info.name.to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    log::warn!("Failed to get clap_audio_port_info.name from plugin instance {}", id);
                    None
                }
            };

            if raw_info.flags.contains(AudioPortFlags::IS_MAIN) {
                if has_main_out_port {
                    log::warn!("Plugin instance {} already has a main output port (at port index {})", id, i)
                } else {
                    has_main_out_port = true;
                }
            }

            Some(AudioPortInfo {
                stable_id: raw_info.id,
                channels: raw_info.channel_count as u16,
                port_type,
                display_name,
            })
        }).collect();

        let main_ports_layout = match (has_main_in_port, has_main_out_port) {
            (true, true) => MainPortsLayout::InOut,
            (true, false) => MainPortsLayout::InOnly,
            (false, true) => MainPortsLayout::OutOnly,
            (false, false) => MainPortsLayout::NoMainPorts,
        };

        Ok(PluginAudioPortsExt { inputs, outputs, main_ports_layout })
    }

    fn parse_note_ports_extension(
        instance: &PluginInstance<ClapHost>,
    ) -> Result<PluginNotePortsExt, String> {
        let id = &*instance.shared_host_data().id;
        log::trace!("clap plugin instance parse note ports extension {}", id);

        if instance.is_active() {
            return Err("Cannot get note ports extension while plugin is active".into());
        }

        let note_ports = match instance.shared_plugin_data().get_extension::<PluginNotePorts>() {
            None => return Ok(PluginNotePortsExt::empty()),
            Some(e) => e,
        };

        let plugin = instance.main_thread_plugin_data();

        let num_in_ports = note_ports.count(&plugin, true);
        let num_out_ports = note_ports.count(&plugin, false);

        let mut buffer = NotePortInfoBuffer::new();

        let inputs: Vec<NotePortInfo> = (0..num_in_ports).filter_map(|i| {
            let raw_info = match note_ports.get(&plugin, i, true, &mut buffer) {
                None => {
                    log::warn!("Error when getting CLAP note port info from plugin instance {}: plugin returned no info for index {}", id, i);
                    return None;
                },
                Some(i) => i
            };

            let display_name = match raw_info.name.to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    log::warn!("Failed to get clap_audio_port_info.name from plugin instance {}", id);
                    None
                }
            };

            Some(NotePortInfo {
                stable_id: raw_info.id,
                supported_dialects: raw_info.supported_dialects,
                preferred_dialect: raw_info.preferred_dialect,
                display_name,
            })
        }).collect();

        let outputs: Vec<NotePortInfo> = (0..num_out_ports).filter_map(|i| {
            let raw_info = match note_ports.get(&plugin, i, false, &mut buffer) {
                None => {
                    log::warn!("Error when getting CLAP note port info from plugin instance {}: plugin returned no info for index {}", id, i);
                    return None;
                },
                Some(i) => i
            };

            let display_name = match raw_info.name.to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    log::warn!("Failed to get clap_audio_port_info.name from plugin instance {}", id);
                    None
                }
            };

            Some(NotePortInfo {
                stable_id: raw_info.id,
                supported_dialects: raw_info.supported_dialects,
                preferred_dialect: raw_info.preferred_dialect,
                display_name,
            })
        }).collect();

        Ok(PluginNotePortsExt { inputs, outputs })
    }
}

impl PluginMainThread for ClapPluginMainThread {
    fn activate(
        &mut self,
        sample_rate: u32,
        min_frames: u32,
        max_frames: u32,
        _coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        let configuration = PluginAudioConfiguration {
            sample_rate: f64::from(sample_rate),
            frames_count_range: min_frames..=max_frames,
        };

        log::trace!("clap plugin instance activate {}", self.id());
        let audio_processor = match self.instance.activate(
            |plugin, shared, _| ClapHostAudioProcessor::new(plugin, shared),
            configuration,
        ) {
            Ok(p) => p,
            Err(e) => return Err(format!("{}", e)),
        };

        Ok(PluginActivatedInfo {
            processor: Box::new(ClapPluginProcessor {
                audio_processor: audio_processor.into(),
                // `PluginHostMainThread` will always call
                // `ClapPluginMainThread::audio_ports_ext()` before activating
                // a plugin, so `audio_port_channels` info should be correct.
                process: ClapProcess::new(&self.audio_port_channels),
            }),
            internal_handle: None,
        })
    }

    fn param_flush(&mut self, in_events: &EventBuffer, out_events: &mut EventBuffer) {
        self.instance.main_thread_host_data_mut().param_flush(in_events, out_events)
    }

    fn collect_save_state(&mut self) -> Result<Option<Vec<u8>>, String> {
        if let Some(state_ext) = self.instance.shared_host_data().state_ext {
            let mut buffer = Vec::new();

            state_ext.save(self.instance.main_thread_plugin_data(), &mut buffer).map_err(|_| {
                format!(
                    "Plugin with ID {} returned error on call to clap_plugin_state.save()",
                    self.id()
                )
            })?;

            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    fn load_save_state(&mut self, state: Vec<u8>) -> Result<(), String> {
        if let Some(state_ext) = self.instance.shared_host_data().state_ext {
            let mut reader = Cursor::new(&state);

            state_ext.load(self.instance.main_thread_plugin_data(), &mut reader).map_err(|_| {
                format!(
                    "Plugin with ID {} returned error on call to clap_plugin_state.load()",
                    self.id()
                )
            })?;

            Ok(())
        } else {
            Err(format!(
                "Could not load state for clap plugin with ID {}: plugin does not implement the \"clap.state\" extension",
                self.id()
            ))
        }
    }

    fn deactivate(&mut self) {
        log::trace!("clap plugin instance deactivate {}", self.id());
        self.instance
            .try_deactivate()
            .expect("Called deactivate() before the plugin's AudioProcessor was dropped");
    }

    fn latency(&self) -> i64 {
        if let Some(latency_ext) = self.instance.shared_host_data().latency_ext {
            latency_ext.get(&mut self.instance.main_thread_plugin_data()) as i64
        } else {
            0
        }
    }

    fn on_main_thread(&mut self) {
        log::trace!("clap plugin instance on_main_thread {}", self.id());

        self.instance.call_on_main_thread_callback();
    }

    fn audio_ports_ext(&mut self) -> Result<PluginAudioPortsExt, String> {
        let res = Self::parse_audio_ports_extension(&self.instance);

        if let Ok(audio_ports) = &res {
            self.audio_port_channels.num_input_ports = audio_ports.inputs.len();
            self.audio_port_channels.num_output_ports = audio_ports.outputs.len();
            self.audio_port_channels.max_input_channels =
                audio_ports.max_input_channels().map(|p| usize::from(p.channels)).unwrap_or(2);
            self.audio_port_channels.max_output_channels =
                audio_ports.max_output_channels().map(|p| usize::from(p.channels)).unwrap_or(2);
        } else {
            self.audio_port_channels = AudioPortChannels::default();
        }

        res
    }

    fn note_ports_ext(&mut self) -> Result<PluginNotePortsExt, String> {
        Self::parse_note_ports_extension(&self.instance)
    }

    // --- Parameters ---------------------------------------------------------------------------------

    fn num_params(&mut self) -> u32 {
        if let Some(params_ext) = self.instance.shared_host_data().params_ext {
            params_ext.count(&self.instance)
        } else {
            0
        }
    }

    fn param_info(&mut self, param_index: usize) -> Result<ext::params::ParamInfo, Box<dyn Error>> {
        if let Some(params_ext) = self.instance.shared_host_data().params_ext {
            let mut data = MaybeUninit::uninit();

            let info = params_ext
                .get_info(&self.instance, param_index as u32, &mut data)
                .ok_or_else(|| format!("Param at index {} does not exist", param_index))?;

            let display_name = if !info.name().is_empty() {
                CStr::from_bytes_with_nul(info.name())?.to_str()?.to_string()
            } else {
                "unnamed".into()
            };
            let module = if !info.module().is_empty() {
                CStr::from_bytes_with_nul(info.module())?.to_str()?.to_string()
            } else {
                String::new()
            };

            Ok(ParamInfo {
                stable_id: ParamID(info.id()),
                flags: ParamInfoFlags::from_bits_truncate(info.flags()),
                display_name,
                module,
                min_value: info.min_value(),
                max_value: info.max_value(),
                default_value: info.default_value(),
                _cookie: info.cookie(),
            })
        } else {
            Err("Plugin does not have any parameters".into())
        }
    }

    fn param_value(&self, param_id: ParamID) -> Result<f64, Box<dyn Error>> {
        if let Some(params_ext) = self.instance.shared_host_data().params_ext {
            params_ext
                .get_value(&self.instance, param_id.0)
                .ok_or_else(|| format!("Param with id {:?} does not exist", param_id).into())
        } else {
            Err("Plugin does not have any parameters".into())
        }
    }

    fn param_value_to_text(
        &self,
        param_id: ParamID,
        value: f64,
        text_buffer: &mut String,
    ) -> Result<(), String> {
        if let Some(params_ext) = self.instance.shared_host_data().params_ext {
            let mut char_buf = [MaybeUninit::uninit(); 256];

            let bytes = match params_ext.value_to_text(
                &self.instance,
                param_id.0,
                value,
                &mut char_buf,
            ) {
                Some(b) => b,
                None => {
                    return Err(format!("Failed to convert parameter value {} on parameter {:?} on plugin {} to a string: plugin returned false on call to value_to_text", value, &param_id, self.id()));
                }
            };

            match core::str::from_utf8(bytes) {
                Ok(s) => {
                    text_buffer.push_str(s);
                    Ok(())
                }
                Err(e) => {
                    Err(format!("Failed to convert parameter value {} on parameter {:?} on plugin {} to a string: {}", value, &param_id, self.id(), e))
                }
            }
        } else {
            Err(String::new())
        }
    }

    fn param_text_to_value(&self, param_id: ParamID, display: &str) -> Option<f64> {
        if let Some(params_ext) = self.instance.shared_host_data().params_ext {
            let c_string = match CString::new(display) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to convert text input to CString in call to param_text_to_value: {}", e);
                    return None;
                }
            };

            params_ext.text_to_value(&self.instance, param_id.0, &c_string)
        } else {
            None
        }
    }

    fn on_timer(&mut self, timer_id: ext::timer::TimerID) {
        if let Some(timer_ext) = self.instance.shared_host_data().timer_ext {
            timer_ext.on_timer(&mut self.instance.main_thread_plugin_data(), TimerId(timer_id.0));
        }
    }

    // --- GUI stuff ---------------------------------------------------------------------------------

    fn supports_gui(&self, floating: bool) -> bool {
        if let (Some(gui), Some(api)) =
            (self.instance.shared_host_data().gui_ext, GuiApiType::default_for_current_platform())
        {
            gui.is_api_supported(&self.instance.main_thread_plugin_data(), api, floating)
        } else {
            false
        }
    }

    fn create_new_floating_gui(
        &mut self,
        suggested_title: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let host = self.instance.main_thread_host_data_mut();

        // TODO: Try to use Wayland on Linux if it is available.
        let api_type =
            GuiApiType::default_for_current_platform().ok_or_else(|| -> Box<dyn Error> {
                "Creating floating GUIs is not supported on this platform".into()
            })?;

        let gui_ext = host.shared.gui_ext.ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not use the GUI extension".into()
        })?;
        let instance = host.instance.as_mut().ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not have an active instance".into()
        })?;

        gui_ext.create(instance, api_type, true).map_err(|e| -> Box<dyn Error> {
            format!("CLAP plugin failed to create a new floating GUI window: {}", e).into()
        })?;

        if let Some(title) = suggested_title {
            match CString::new(title.to_string()) {
                Ok(title) => {
                    gui_ext.suggest_title(instance, &title);
                }
                Err(e) => {
                    log::warn!("Failed to convert suggested title {} to CString: {}", title, e);
                }
            }
        }

        Ok(())
    }

    fn create_new_embedded_gui(
        &mut self,
        scale: Option<f64>,
        size: Option<GuiSize>,
        parent_window: RawWindowHandle,
    ) -> Result<EmbeddedGuiInfo, Box<dyn Error>> {
        let host = self.instance.main_thread_host_data_mut();

        // TODO: Try to use Wayland on Linux if it is available.
        let api_type =
            GuiApiType::default_for_current_platform().ok_or_else(|| -> Box<dyn Error> {
                "Creating embedded GUIs is not supported on this platform".into()
            })?;

        let gui_ext = host.shared.gui_ext.ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not use the GUI extension".into()
        })?;
        let instance = host.instance.as_mut().ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not have an active instance".into()
        })?;

        let window = clack_extensions::gui::Window::from_raw_window_handle(parent_window)
            .ok_or_else(|| -> Box<dyn Error> {
                "Creating embedded GUIs is not supported on this platform".into()
            })?;

        gui_ext.create(instance, api_type, false).map_err(|e| -> Box<dyn Error> {
            format!("CLAP plugin failed to create a new embedded GUI: {}", e).into()
        })?;

        if let Some(scale) = scale {
            if let Err(e) = gui_ext.set_scale(instance, scale) {
                log::warn!(
                    "CLAP plugin failed to set the scale to {} in its new embedded GUI: {}",
                    scale,
                    e
                );
            }
        }

        let resizable = gui_ext.can_resize(instance);

        if resizable {
            if let Some(size) = size {
                if let Err(e) = gui_ext.set_size(instance, size) {
                    log::warn!("CLAP plugin failed to set its width and height to {:?} for its new embedded GUI: {}", size, e);
                }
            }
        }

        let working_size = if let Some(working_size) = gui_ext.get_size(instance) {
            working_size
        } else {
            gui_ext.destroy(instance);
            return Err("CLAP plugin failed to return the size of its new embedded GUI".into());
        };

        let resize_hints = if resizable { gui_ext.get_resize_hints(instance) } else { None };

        if let Err(e) = gui_ext.set_parent(instance, &window) {
            gui_ext.destroy(instance);
            return Err(format!(
                "CLAP plugin failed to set its embedded GUI to the given parent window: {}",
                e
            )
            .into());
        }

        Ok(EmbeddedGuiInfo { size: working_size, resizable, resize_hints })
    }

    fn destroy_gui(&mut self) {
        let host = self.instance.main_thread_host_data_mut();
        if let Some(gui_ext) = host.shared.gui_ext {
            if let Some(instance) = host.instance.as_mut() {
                gui_ext.destroy(instance);
            } else {
                log::warn!(
                    "Host called `destroy_gui()`, but CLAP plugin does have an active instance"
                );
            }
        } else {
            log::warn!(
                "Host called `destroy_gui()`, but CLAP plugin does not use the GUI extension"
            );
        }
    }

    fn adjust_gui_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        let host = self.instance.main_thread_host_data_mut();
        if let Some(gui_ext) = host.shared.gui_ext {
            if let Some(instance) = host.instance.as_mut() {
                return gui_ext.adjust_size(instance, size);
            } else {
                log::warn!(
                    "Host called `adjust_gui_size()`, but CLAP plugin does have an active instance"
                );
            }
        } else {
            log::warn!(
                "Host called `adjust_gui_size()`, but CLAP plugin does not use the GUI extension"
            );
        }

        None
    }

    fn set_gui_size(&mut self, size: GuiSize) -> Result<(), Box<dyn Error>> {
        let host = self.instance.main_thread_host_data_mut();
        let gui_ext = host.shared.gui_ext.ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not use the GUI extension".into()
        })?;
        let instance = host.instance.as_mut().ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not have an active instance".into()
        })?;

        gui_ext.set_size(instance, size).map_err(|e| -> Box<dyn Error> {
            format!("CLAP plugin failed to set the size of its GUI to {:?}: {}", size, e).into()
        })
    }

    fn show_gui(&mut self) -> Result<(), Box<dyn Error>> {
        let host = self.instance.main_thread_host_data_mut();
        let gui_ext = host.shared.gui_ext.ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not use the GUI extension".into()
        })?;
        let instance = host.instance.as_mut().ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not have an active instance".into()
        })?;

        gui_ext.show(instance).map_err(|e| -> Box<dyn Error> {
            format!("CLAP plugin failed to show its GUI: {}", e).into()
        })
    }

    /// Hide the currently active GUI
    ///
    /// By default this does nothing.
    fn hide_gui(&mut self) -> Result<(), Box<dyn Error>> {
        let host = self.instance.main_thread_host_data_mut();
        let gui_ext = host.shared.gui_ext.ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not use the GUI extension".into()
        })?;
        let instance = host.instance.as_mut().ok_or_else(|| -> Box<dyn Error> {
            "CLAP plugin does not have an active instance".into()
        })?;

        gui_ext.show(instance).map_err(|e| -> Box<dyn Error> {
            format!("CLAP plugin failed to hide its GUI: {}", e).into()
        })
    }
}

struct ClapPluginProcessor {
    audio_processor: PluginAudioProcessor<ClapHost>,
    process: ClapProcess,
}

impl PluginProcessor for ClapPluginProcessor {
    fn start_processing(&mut self) -> Result<(), Box<dyn Error>> {
        log::trace!(
            "clap plugin instance start_processing {}",
            &*self.audio_processor.shared_host_data().id
        );

        self.audio_processor.start_processing().map_err(|e| e.into())
    }

    fn stop_processing(&mut self) {
        log::trace!(
            "clap plugin instance stop_processing {}",
            &*self.audio_processor.shared_host_data().id
        );

        if self.audio_processor.is_started() {
            if let Err(e) = self.audio_processor.stop_processing() {
                log::error!("{}", e);
            }
        }
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        in_events: &EventBuffer,
        out_events: &mut EventBuffer,
    ) -> ProcessStatus {
        let (audio_in, mut audio_out) = self.process.update_buffers(buffers);

        let mut in_events = InputEvents::from_buffer(in_events);
        let mut out_events = OutputEvents::from_buffer(out_events);

        let res = {
            // In debug mode, borrow all of the atomic ref cells to properly use the
            // safety checks, since external plugins just use the raw pointer to each
            // buffer.
            //#[cfg(debug_assertions)]
            let (mut input_refs_f32, mut input_refs_f64, mut output_refs_f32, mut output_refs_f64) = {
                let mut input_refs_f32: SmallVec<[AtomicRef<'_, BufferInner<f32>>; 32]> =
                    SmallVec::new();
                let mut input_refs_f64: SmallVec<[AtomicRef<'_, BufferInner<f64>>; 32]> =
                    SmallVec::new();
                let mut output_refs_f32: SmallVec<[AtomicRefMut<'_, BufferInner<f32>>; 32]> =
                    SmallVec::new();
                let mut output_refs_f64: SmallVec<[AtomicRefMut<'_, BufferInner<f64>>; 32]> =
                    SmallVec::new();

                for in_port in buffers.audio_in.iter() {
                    match &in_port._raw_channels {
                        RawAudioChannelBuffers::F32(buffers) => {
                            for b in buffers.iter() {
                                input_refs_f32.push(b.borrow());
                            }
                        }
                        RawAudioChannelBuffers::F64(buffers) => {
                            for b in buffers.iter() {
                                input_refs_f64.push(b.borrow());
                            }
                        }
                    }
                }

                for out_port in buffers.audio_out.iter() {
                    match &out_port._raw_channels {
                        RawAudioChannelBuffers::F32(buffers) => {
                            for b in buffers.iter() {
                                output_refs_f32.push(b.borrow_mut());
                            }
                        }
                        RawAudioChannelBuffers::F64(buffers) => {
                            for b in buffers.iter() {
                                output_refs_f64.push(b.borrow_mut());
                            }
                        }
                    }
                }

                (input_refs_f32, input_refs_f64, output_refs_f32, output_refs_f64)
            };

            // TODO: handle transport & timer
            let res = self
                .audio_processor
                .as_started_mut()
                .expect("Audio Processor is not started")
                .process(
                    &audio_in,
                    &mut audio_out,
                    &mut in_events,
                    &mut out_events,
                    proc_info.steady_time,
                    Some(proc_info.frames),
                    None,
                );

            // TODO: Sync audio output constant flags.

            //#[cfg(debug_assertions)]
            {
                input_refs_f32.clear();
                input_refs_f64.clear();
                output_refs_f32.clear();
                output_refs_f64.clear();
            }

            res
        };

        use clack_host::process::ProcessStatus::*;
        match res {
            Err(_) => ProcessStatus::Error,
            Ok(Continue) => ProcessStatus::Continue,
            Ok(ContinueIfNotQuiet) => ProcessStatus::ContinueIfNotQuiet,
            Ok(Tail) => ProcessStatus::Tail,
            Ok(Sleep) => ProcessStatus::Sleep,
        }
    }

    fn param_flush(&mut self, in_events: &EventBuffer, out_events: &mut EventBuffer) {
        self.audio_processor.audio_processor_host_data_mut().param_flush(in_events, out_events)
    }
}
