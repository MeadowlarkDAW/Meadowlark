//! BIG TODO: Have the entire engine run in a separate process for
//! crash protection from plugins.

use std::time::{Duration, Instant};

use dropseed::plugin_api::{HostInfo, ParamID, PluginInstanceID};
use dropseed::{
    engine::{
        modify_request::{ConnectEdgeReq, EdgeReqPortID, ModifyGraphRequest, PluginIDReq},
        ActivateEngineSettings, ActivatedEngineInfo, DSEngineMainThread, EngineDeactivatedStatus,
        EngineSettings, OnIdleEvent, PluginStatus,
    },
    graph::PortType,
    plugin_api::DSPluginSaveState,
};

use crate::backend::resource_loader::ResourceLoader;
use crate::backend::sample_browser_plug::{
    SampleBrowserPlugFactory, SampleBrowserPlugHandle, SAMPLE_BROWSER_PLUG_RDN,
};
use crate::backend::system_io::SystemIOStreamHandle;

use super::BrowserPanelState;

// TODO: Have these be configurable.
const MIN_FRAMES: u32 = 1;
const MAX_FRAMES: u32 = 512;
const GRAPH_IN_CHANNELS: u16 = 2;
const GRAPH_OUT_CHANNELS: u16 = 2;

static RESOURCE_COLLECT_INTERVAL: Duration = Duration::from_secs(3);

pub struct EngineHandle {
    pub ds_engine: DSEngineMainThread,
    pub activated_state: Option<ActivatedEngineState>,

    next_timer_instant: Instant,
    next_resource_collect_instant: Instant,

    system_io_stream_handle: SystemIOStreamHandle,
}

impl EngineHandle {
    pub fn new(browser_panel_state: &BrowserPanelState) -> Self {
        // TODO: Use rainout instead of cpal once it's ready.
        // TODO: Load settings from a save file rather than spawning
        // a stream with default settings.
        let mut system_io_stream_handle =
            crate::backend::system_io::temp_spawn_cpal_default_output_only().unwrap();

        let (mut ds_engine, first_timer_instant, internal_plugins_scan_res) =
            DSEngineMainThread::new(
                HostInfo::new(
                    "Meadowlark".into(),                   // host name
                    env!("CARGO_PKG_VERSION").into(),      // host version
                    Some("Meadowlark".into()),             // vendor
                    Some("https://meadowlark.app".into()), // url
                ),
                EngineSettings::default(),
                vec![Box::new(SampleBrowserPlugFactory)], // list of internal plugins
            );

        log::info!("{:?}", &internal_plugins_scan_res);

        let (engine_info, ds_engine_audio_thread) = ds_engine
            .activate_engine(ActivateEngineSettings {
                sample_rate: system_io_stream_handle.sample_rate(),
                min_frames: MIN_FRAMES,
                max_frames: MAX_FRAMES,
                num_audio_in_channels: GRAPH_IN_CHANNELS,
                num_audio_out_channels: GRAPH_OUT_CHANNELS,
                ..Default::default()
            })
            .unwrap();

        system_io_stream_handle.on_engine_activated(ds_engine_audio_thread);

        // Add a sample browser plugin to the graph, and connect it directly
        // to the graph output.
        let mut sample_browser_plug_key = None;
        for res in internal_plugins_scan_res.iter() {
            if let Ok(res) = res {
                if res.rdn == SAMPLE_BROWSER_PLUG_RDN {
                    sample_browser_plug_key = Some(res.clone());
                }
            }
        }
        let sample_browser_plug_key = sample_browser_plug_key.unwrap();
        let graph_out_id = engine_info.graph_out_id.clone();
        let mut res = ds_engine
            .modify_graph(ModifyGraphRequest {
                add_plugin_instances: vec![DSPluginSaveState::new_with_default_state(
                    sample_browser_plug_key,
                )],
                remove_plugin_instances: vec![],
                connect_new_edges: vec![
                    ConnectEdgeReq {
                        edge_type: PortType::Audio,
                        src_plugin_id: PluginIDReq::Added(0),
                        dst_plugin_id: PluginIDReq::Existing(graph_out_id.clone()),
                        src_port_id: EdgeReqPortID::Main,
                        src_port_channel: 0,
                        dst_port_id: EdgeReqPortID::Main,
                        dst_port_channel: 0,
                        check_for_cycles: false,
                        log_error_on_fail: true,
                    },
                    ConnectEdgeReq {
                        edge_type: PortType::Audio,
                        src_plugin_id: PluginIDReq::Added(0),
                        dst_plugin_id: PluginIDReq::Existing(graph_out_id),
                        src_port_id: EdgeReqPortID::Main,
                        src_port_channel: 1,
                        dst_port_id: EdgeReqPortID::Main,
                        dst_port_channel: 1,
                        check_for_cycles: false,
                        log_error_on_fail: true,
                    },
                ],
                disconnect_edges: vec![],
            })
            .unwrap();

        let sample_browser_plug_res = res.new_plugins.remove(0);
        let sample_browser_plug_id = sample_browser_plug_res.plugin_id;
        let sample_browser_plug_host = ds_engine.plugin_host_mut(&sample_browser_plug_id).unwrap();
        let sample_browser_plug_params = sample_browser_plug_host.param_list().to_owned();
        let sample_browser_plug_handle =
            if let PluginStatus::Activated(status) = sample_browser_plug_res.status {
                *(status.internal_handle.unwrap().downcast::<SampleBrowserPlugHandle>().unwrap())
            } else {
                panic!("Sample browser plugin failed to activate");
            };
        sample_browser_plug_host
            .set_param_value(
                sample_browser_plug_params[0],
                f64::from(browser_panel_state.volume_normalized),
            )
            .unwrap();

        let resource_loader = ResourceLoader::new(system_io_stream_handle.sample_rate());

        let activated_state = ActivatedEngineState {
            engine_info,
            sample_browser_plug_id,
            sample_browser_plug_params,
            sample_browser_plug_handle,
            resource_loader,
        };

        Self {
            ds_engine,
            activated_state: Some(activated_state),
            next_timer_instant: first_timer_instant,
            next_resource_collect_instant: Instant::now() + RESOURCE_COLLECT_INTERVAL,
            system_io_stream_handle,
        }
    }

    pub fn poll_engine(&mut self) {
        let now = Instant::now();
        if now >= self.next_timer_instant {
            let (mut events, next_timer_instant) = self.ds_engine.on_timer();
            self.next_timer_instant = next_timer_instant;

            for event in events.drain(..) {
                self.on_engine_event(event);
            }
        }
        if now >= self.next_resource_collect_instant {
            if let Some(activated_state) = &mut self.activated_state {
                activated_state.resource_loader.collect();
            }

            self.next_resource_collect_instant = now + RESOURCE_COLLECT_INTERVAL;
        }
    }

    fn on_engine_event(&mut self, event: OnIdleEvent) {
        match event {
            // The plugin's parameters have been modified via the plugin's custom
            // GUI.
            //
            // Only the parameters which have changed will be returned in this
            // field.
            OnIdleEvent::PluginParamsModified { plugin_id, modified_params } => {}

            // The plugin requested the app to resize its gui to the given size.
            //
            // This event will only be sent if using an embedded window for the
            // plugin GUI.
            OnIdleEvent::PluginRequestedToResizeGui { plugin_id, size } => {}

            // The plugin requested the app to show its GUI.
            //
            // This event will only be sent if using an embedded window for the
            // plugin GUI.
            OnIdleEvent::PluginRequestedToShowGui { plugin_id } => {}

            // The plugin requested the app to hide its GUI.
            //
            // Note that hiding the GUI is not the same as destroying the GUI.
            // Hiding only hides the window content, it does not free the GUI's
            // resources.  Yet it may be a good idea to stop painting timers
            // when a plugin GUI is hidden.
            //
            // This event will only be sent if using an embedded window for the
            // plugin GUI.
            OnIdleEvent::PluginRequestedToHideGui { plugin_id } => {}

            // Sent when the plugin closed its own GUI by its own means. UI should
            // be updated accordingly so that the user could open the UI again.
            //
            // If `was_destroyed` is `true`, then the app must call
            // `PluginHostMainThread::destroy_gui()` to acknowledge the gui
            // destruction.
            OnIdleEvent::PluginGuiClosed { plugin_id, was_destroyed } => {}

            // Sent when the plugin changed the resize hint information on how
            // to resize its GUI.
            //
            // This event will only be sent if using an embedded window for the
            // plugin GUI.
            OnIdleEvent::PluginChangedGuiResizeHints { plugin_id, resize_hints } => {}

            // The plugin has updated its list of parameters.
            OnIdleEvent::PluginUpdatedParameterList { plugin_id, status } => {}

            // Sent whenever a plugin becomes activated after being deactivated or
            // when the plugin restarts.
            //
            // Make sure your UI updates the port configuration on this plugin, as
            // well as any custom handles.
            OnIdleEvent::PluginActivated { plugin_id, status } => {}

            // Sent whenever a plugin has been deactivated. When a plugin is
            // deactivated, you cannot access any of its methods until it is
            // reactivated.
            OnIdleEvent::PluginDeactivated { plugin_id, status } => {}

            // Sent whenever the engine has been deactivated, whether gracefully or
            // because of a crash.
            OnIdleEvent::EngineDeactivated(status) => {
                self.activated_state = None;
                self.system_io_stream_handle.on_engine_deactivated();

                match status {
                    EngineDeactivatedStatus::DeactivatedGracefully => {
                        log::info!("Engine was deactivated gracefully");
                    }
                    EngineDeactivatedStatus::EngineCrashed(e) => {
                        log::error!("Engine crashed: {}", e);
                    }
                }
            }
        }
    }
}

pub struct ActivatedEngineState {
    pub engine_info: ActivatedEngineInfo,
    pub resource_loader: ResourceLoader,

    pub sample_browser_plug_id: PluginInstanceID,
    pub sample_browser_plug_params: Vec<ParamID>,
    pub sample_browser_plug_handle: SampleBrowserPlugHandle,
}
