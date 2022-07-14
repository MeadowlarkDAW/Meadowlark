use crossbeam::channel::Receiver;
use dropseed::plugin::PluginSaveState;
use dropseed::plugins::sample_browser::{
    SampleBrowserPlugFactory, SampleBrowserPlugHandle, SAMPLE_BROWSER_PLUG_RDN,
};
use dropseed::plugins::test_sine::{TestSineStereoFactory, TEST_SINE_STEREO_RDN};
use dropseed::resource_loader::{PcmKey, ResampleQuality};
use dropseed::{
    transport::TransportHandle, ActivateEngineSettings, ActivatePluginError, DSEngineEvent,
    DSEngineHandle, DSEngineRequest, EdgeReq, EdgeReqPortID, EngineActivatedInfo,
    EngineDeactivatedInfo, HostInfo, ModifyGraphRequest, ModifyGraphRes, ParamID,
    ParamModifiedInfo, PluginActivationStatus, PluginEvent, PluginHandle, PluginIDReq,
    PluginInstanceID, PluginScannerEvent, PortType, RescanPluginDirectoriesRes, ResourceLoader,
};
use fnv::FnvHashMap;
use meadowlark_core_types::{MusicalTime, SampleRate};
use smallvec::SmallVec;
use std::error::Error;
use std::{fmt::Debug, path::PathBuf};
use vizia::prelude::*;

use crate::backend::system_io::{self, SystemIOStreamHandle};

mod browser;
mod channel;
mod clip;
mod core_types;
mod event;
mod hrack_effect;
mod lane_states;
mod panel;
mod timeline_grid;

pub use browser::*;
pub use channel::*;
pub use clip::*;
pub use core_types::*;
pub use event::*;
pub use hrack_effect::*;
pub use lane_states::*;
pub use panel::*;
pub use timeline_grid::*;

// TODO: Have these be configurable.
const MIN_FRAMES: u32 = 1;
const MAX_FRAMES: u32 = 512;
const GRAPH_IN_CHANNELS: u16 = 2;
const GRAPH_OUT_CHANNELS: u16 = 2;

pub struct EngineHandles {
    ds_handle: DSEngineHandle,

    activated_info: Option<ActivatedEngineInfo>,
    sample_browser_plug_handle: Option<PluginHandle>,
}

pub struct ActivatedEngineInfo {
    /// The ID for the input to the audio graph. Use this to connect any
    /// plugins to system inputs.
    pub graph_in_node_id: PluginInstanceID,

    /// The ID for the output to the audio graph. Use this to connect any
    /// plugins to system outputs.
    pub graph_out_node_id: PluginInstanceID,

    pub transport_handle: TransportHandle,

    pub sample_rate: SampleRate,
    pub min_frames: u32,
    pub max_frames: u32,
    pub num_audio_in_channels: u16,
    pub num_audio_out_channels: u16,
}

#[derive(Debug, Lens)]
pub enum NotificationLogType {
    Error(String),
    Info(String),
}

#[derive(Lens)]
pub struct ActiveEngineInfo {
    /// The ID for the input to the audio graph. Use this to connect any
    /// plugins to system inputs.
    #[lens(ignore)]
    pub graph_in_node_id: PluginInstanceID,

    /// The ID for the output to the audio graph. Use this to connect any
    /// plugins to system outputs.
    #[lens(ignore)]
    pub graph_out_node_id: PluginInstanceID,

    #[lens(ignore)]
    pub transport_handle: TransportHandle,

    pub sample_rate: SampleRate,
    pub min_frames: u32,
    pub max_frames: u32,
    pub num_audio_in_channels: u16,
    pub num_audio_out_channels: u16,
}

#[derive(Lens)]
pub struct UiData {
    pub state: UiState,

    /// This contains all of the text for any notifications (errors or otherwise)
    /// that are being displayed to the user.
    ///
    /// The UI may mutate this directly without an event.
    pub notification_log: Vec<NotificationLogType>,

    /// True if a backend engine is currently running, false if not.
    ///
    /// Nothing except the settings menu can be accessed when this is false.
    pub engine_running: bool,

    #[lens(ignore)]
    pub resource_loader: ResourceLoader,

    #[lens(ignore)]
    last_clicked_browser_file: Option<PathBuf>,

    #[lens(ignore)]
    system_io_stream_handle: Option<SystemIOStreamHandle>,

    #[lens(ignore)]
    engine_handles: Option<(EngineHandles, Receiver<DSEngineEvent>)>,
}

impl UiData {
    // Create some dummy state for now
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // This is temporary. Eventually we will have a more sophisticated and
        // configurable system using `rainout`.
        let system_io_stream_handle = system_io::temp_spawn_cpal_default_output_only()?;
        let sample_rate = system_io_stream_handle.sample_rate();

        let resource_loader = ResourceLoader::new(sample_rate);

        // Fill with dummy state for now.
        let mut app_data = UiData {
            state: UiState {
                channels: vec![
                    ChannelState {
                        name: String::from("Master"),
                        selected: false,
                        color: Color::from("#D4D5D5").into(),
                        subchannels: vec![1, 5],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Drum Group"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![2, 3, 4],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Kick"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Snare"),
                        selected: true,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Hat"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Spicy Synth"),
                        selected: false,
                        color: Color::from("#EA716C").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                ],
                clips: vec![ClipState {
                    name: String::from("Drum Group 1"),
                    channel: 1,
                    timeline_start: ClipStart::NotInTimeline,
                    length: MusicalTime::from_beats(4).into(),
                    type_: ClipType::Automation(AutomationClipState {}),
                }],
                timeline_grid: TimelineGridState {
                    horizontal_zoom_level: 1.0,
                    vertical_zoom_level: 1.0,
                    left_start: MusicalTime::from_beats(0).into(),
                    top_start: 0.0,
                    lane_height: 1.0,
                    lane_states: LaneStates::new(vec![
                        LaneState {
                            name: Some(String::from("Track 1")),
                            color: Some(Color::from("#EDE171").into()),
                            height: Some(2.0),
                            disabled: false,
                            selected: false,
                        },
                        LaneState {
                            name: Some(String::from("Track 2")),
                            color: Some(Color::from("#EDE171").into()),
                            height: None,
                            disabled: false,
                            selected: false,
                        },
                        LaneState {
                            name: Some(String::from("Track 3")),
                            color: Some(Color::from("#EA716C").into()),
                            height: None,
                            disabled: false,
                            selected: false,
                        },
                    ]),
                    project_length: MusicalTime::from_beats(16).into(),
                    used_lanes: 0,
                },
                browser: BrowserState::default(),
                panels: PanelState {
                    channel_rack_orientation: ChannelRackOrientation::Horizontal,
                    hide_clips: false,
                    hide_piano_roll: false,
                    browser_width: 200.0,
                    hide_browser: false,
                },
                dragging_channel: None,
            },
            resource_loader,
            notification_log: Vec::new(),
            engine_running: false,
            system_io_stream_handle: Some(system_io_stream_handle),
            last_clicked_browser_file: None,
            engine_handles: None,
        };

        app_data.activate_engine();

        Ok(app_data)
    }

    pub fn activate_engine(&mut self) {
        if let Some(system_io_stream_handle) = &mut self.system_io_stream_handle {
            let (mut engine_handle, engine_rx) = DSEngineHandle::new(
                HostInfo::new(
                    String::from("RustyDAW integration test"),
                    String::from("0.1.0"),
                    None,
                    None,
                ),
                vec![Box::new(SampleBrowserPlugFactory), Box::new(TestSineStereoFactory)],
            );

            log::debug!("{:?}", &engine_handle.internal_plugins_res);

            let sample_rate = system_io_stream_handle.sample_rate();

            engine_handle.send(DSEngineRequest::ActivateEngine(Box::new(ActivateEngineSettings {
                sample_rate,
                min_frames: MIN_FRAMES,
                max_frames: MAX_FRAMES,
                num_audio_in_channels: GRAPH_IN_CHANNELS,
                num_audio_out_channels: GRAPH_OUT_CHANNELS,
                ..ActivateEngineSettings::default()
            })));

            engine_handle.send(DSEngineRequest::RescanPluginDirectories);

            self.engine_handles = Some((
                EngineHandles {
                    ds_handle: engine_handle,
                    activated_info: None,
                    sample_browser_plug_handle: None,
                },
                engine_rx,
            ));
        } else {
            log::warn!("Cannot activate engine until a system IO stream is started");
        }
    }

    pub fn poll_engine(&mut self) {
        let Self { state, system_io_stream_handle, engine_handles, resource_loader, .. } = self;

        if let Some((engine_handles, engine_rx)) = engine_handles {
            //let EngineHandles { handle, rx, activated_info, sample_browser_plug_handle } = engine_handle;

            for msg in engine_rx.try_iter() {
                match msg {
                    // TODO: Hint to the compiler that this is by far the most likely event?
                    DSEngineEvent::Plugin(PluginEvent::ParamsModified {
                        plugin_id,
                        modified_params,
                    }) => {
                        state.on_plugin_params_modified(plugin_id, modified_params);
                    }
                    // TODO: Hint to the compiler that this is the next most likely event?
                    DSEngineEvent::AudioGraphModified(event) => {
                        state.on_audio_graph_modified(event, engine_handles);
                    }
                    DSEngineEvent::Plugin(PluginEvent::Activated {
                        plugin_id,
                        new_handle,
                        new_param_values,
                    }) => {
                        state.on_plugin_activated(plugin_id, new_handle, new_param_values);
                    }
                    DSEngineEvent::Plugin(PluginEvent::Deactivated { plugin_id, status }) => {
                        state.on_plugin_deactivated(plugin_id, status);
                    }
                    DSEngineEvent::EngineDeactivated(event) => {
                        self.engine_running = false;
                        state.on_engine_deactivated(event, engine_handles, system_io_stream_handle);
                    }
                    DSEngineEvent::EngineActivated(event) => {
                        self.engine_running = true;
                        state.on_engine_activated(event, engine_handles, system_io_stream_handle);
                    }
                    DSEngineEvent::AudioGraphCleared => {
                        state.on_audio_graph_cleared();
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::ClapScanPathAdded(path)) => {
                        state.on_clap_scan_path_added(path);
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::ClapScanPathRemoved(path)) => {
                        state.on_clap_scan_path_removed(path);
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::RescanFinished(event)) => {
                        state.on_plugin_scanner_rescan_finished(event);
                    }
                    unkown_event => {
                        log::warn!("{:?}", unkown_event);
                    }
                }
            }
        }

        // Clean up loaded resources that are no longer being used.
        //
        // TODO: Only call this periodically (i.e. every 3 seconds or so), because
        // this can get expensive when a lot of resources are loaded in the project.
        resource_loader.collect();
    }
}

impl Model for UiData {
    // Update the program layer here
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|program_event, _| match program_event {
            UiEvent::PollEngine => {
                self.poll_engine();
            }
            UiEvent::SaveProject => {
                //let save_state = serde_json::to_string(&self.state).unwrap();
                //std::fs::write("project.json", save_state).unwrap();
            }
            UiEvent::LoadProject => {
                //let save_state = std::fs::read_to_string("project.json").unwrap();
                //let project_state = serde_json::from_str(&save_state).unwrap();
                //self.state = project_state;
            }
            UiEvent::BrowserFileClicked(path) => {
                if let Some((engine_handles, _)) = &mut self.engine_handles {
                    if let Some(browser_plug_handle) =
                        &mut engine_handles.sample_browser_plug_handle
                    {
                        let browser_plug_handle = browser_plug_handle
                            .internal
                            .as_mut()
                            .unwrap()
                            .downcast_mut::<SampleBrowserPlugHandle>()
                            .unwrap();

                        // TODO: Only play audio files.
                        let already_loaded =
                            if let Some(last_path) = &self.last_clicked_browser_file {
                                last_path == path
                            } else {
                                false
                            };

                        if already_loaded {
                            browser_plug_handle.replay_sample();
                        } else {
                            let (pcm, res) = self.resource_loader.pcm_loader.load(&PcmKey {
                                path: path.clone(),
                                resample_to_project_sr: true,
                                quality: ResampleQuality::Linear,
                            });

                            match res {
                                Ok(()) => {
                                    browser_plug_handle.play_sample(pcm);
                                    self.last_clicked_browser_file = Some(path.clone());
                                }
                                Err(e) => {
                                    log::error!("Failed to load pcm resource: {}", e);
                                    self.last_clicked_browser_file = None;
                                }
                            }
                        }
                    }
                }
            }
            UiEvent::BrowserFileStop() => {
                if let Some((engine_handles, _)) = &mut self.engine_handles {
                    if let Some(browser_plug_handle) =
                        &mut engine_handles.sample_browser_plug_handle
                    {
                        let browser_plug_handle = browser_plug_handle
                            .internal
                            .as_mut()
                            .unwrap()
                            .downcast_mut::<SampleBrowserPlugHandle>()
                            .unwrap();

                        self.last_clicked_browser_file = None;
                        browser_plug_handle.stop();
                    }
                }
            }
            _ => {}
        });

        self.state.event(cx, event);
    }
}

#[derive(Debug, Lens, Clone)]
pub struct UiState {
    /// A "channel" refers to a mixer channel.
    ///
    /// This also contains the state of all clips.
    pub channels: Vec<ChannelState>,

    // Index of channel being dragged
    pub dragging_channel: Option<usize>,

    pub clips: Vec<ClipState>,

    /// The state of the timeline grid.
    ///
    /// (This does not contain the state of the clips.)
    pub timeline_grid: TimelineGridState,

    pub browser: BrowserState,

    /// State of the UI panels.
    ///
    /// This is visual state that is used by the UI and must be serialized.
    pub panels: PanelState,
}

impl UiState {
    /// Sent whenever the engine is deactivated.
    ///
    /// The DSEngineAudioThread sent in a previous EngineActivated event is now
    /// invalidated. Please drop it and wait for a new EngineActivated event to
    /// replace it.
    ///
    /// To keep using the audio graph, you must reactivate the engine with
    /// `DSEngineRequest::ActivateEngine`, and then restore the audio graph
    /// from an existing save state if you wish using
    /// `DSEngineRequest::RestoreFromSaveState`.
    fn on_engine_deactivated(
        &mut self,
        event: EngineDeactivatedInfo,
        engine_handles: &mut EngineHandles,
        system_io_stream_handle: &mut Option<SystemIOStreamHandle>,
    ) {
        engine_handles.activated_info = None;
        engine_handles.sample_browser_plug_handle = None;

        if let Some(system_io_stream_handle) = system_io_stream_handle.as_mut() {
            system_io_stream_handle.engine_deactivated();
        }

        // TODO
    }

    /// This message is sent whenever the engine successfully activates.
    fn on_engine_activated(
        &mut self,
        event: EngineActivatedInfo,
        engine_handles: &mut EngineHandles,
        system_io_stream_handle: &mut Option<SystemIOStreamHandle>,
    ) {
        engine_handles.activated_info = Some(ActivatedEngineInfo {
            graph_in_node_id: event.graph_in_node_id.clone(),
            graph_out_node_id: event.graph_out_node_id.clone(),
            transport_handle: event.transport_handle,
            sample_rate: event.sample_rate,
            min_frames: event.min_frames,
            max_frames: event.max_frames,
            num_audio_in_channels: event.num_audio_in_channels,
            num_audio_out_channels: event.num_audio_out_channels,
        });

        // Collect the keys for the internal plugins.
        let mut sample_browser_plug_key = None;
        let mut test_sine_stereo_key = None;
        for p in engine_handles.ds_handle.internal_plugins_res.iter() {
            if let Ok(key) = p {
                if &key.rdn == SAMPLE_BROWSER_PLUG_RDN {
                    sample_browser_plug_key = Some(key.clone());
                } else if &key.rdn == TEST_SINE_STEREO_RDN {
                    test_sine_stereo_key = Some(key.clone());
                }
            }
        }
        let sample_browser_plug_key = sample_browser_plug_key.unwrap();
        let test_sine_stereo_key = test_sine_stereo_key.unwrap();

        system_io_stream_handle.as_mut().unwrap().engine_activated(event.audio_thread);

        // Add the sample-browser plugin and connect it directly to the output.
        engine_handles.ds_handle.send(DSEngineRequest::ModifyGraph(ModifyGraphRequest {
            add_plugin_instances: vec![
                PluginSaveState::new_with_default_preset(sample_browser_plug_key),
                //PluginSaveState::new_with_default_preset(test_sine_stereo_key),
            ],
            remove_plugin_instances: vec![],
            connect_new_edges: vec![
                EdgeReq {
                    edge_type: PortType::Audio,
                    src_plugin_id: PluginIDReq::Added(0),
                    dst_plugin_id: PluginIDReq::Existing(event.graph_out_node_id.clone()),
                    src_port_id: EdgeReqPortID::Main,
                    src_port_channel: 0,
                    dst_port_id: EdgeReqPortID::Main,
                    dst_port_channel: 0,
                    log_error_on_fail: true,
                },
                EdgeReq {
                    edge_type: PortType::Audio,
                    src_plugin_id: PluginIDReq::Added(0),
                    dst_plugin_id: PluginIDReq::Existing(event.graph_out_node_id.clone()),
                    src_port_id: EdgeReqPortID::Main,
                    src_port_channel: 1,
                    dst_port_id: EdgeReqPortID::Main,
                    dst_port_channel: 1,
                    log_error_on_fail: true,
                },
                /*
                EdgeReq {
                    edge_type: PortType::Audio,
                    src_plugin_id: PluginIDReq::Added(1),
                    dst_plugin_id: PluginIDReq::Existing(event.graph_out_node_id.clone()),
                    src_port_id: EdgeReqPortID::Main,
                    src_port_channel: 0,
                    dst_port_id: EdgeReqPortID::Main,
                    dst_port_channel: 0,
                    log_error_on_fail: true,
                },
                EdgeReq {
                    edge_type: PortType::Audio,
                    src_plugin_id: PluginIDReq::Added(1),
                    dst_plugin_id: PluginIDReq::Existing(event.graph_out_node_id.clone()),
                    src_port_id: EdgeReqPortID::Main,
                    src_port_channel: 1,
                    dst_port_id: EdgeReqPortID::Main,
                    dst_port_channel: 1,
                    log_error_on_fail: true,
                },
                */
            ],
            disconnect_edges: vec![],
        }));
    }

    /// When this message is received, it means that the audio graph is starting
    /// the process of restoring from a save state.
    ///
    /// Reset your UI as if you are loading up a project for the first time, and
    /// wait for the `AudioGraphModified` event to repopulate the UI.
    ///
    /// If the audio graph is in an invalid state as a result of restoring from
    /// the save state, then the `EngineDeactivated` event will be sent instead.
    fn on_audio_graph_cleared(&mut self) {
        // TODO
    }

    /// This message is sent whenever the audio graph has been modified.
    ///
    /// Be sure to update your UI from this new state.
    fn on_audio_graph_modified(
        &mut self,
        mut event: ModifyGraphRes,
        engine_handles: &mut EngineHandles,
    ) {
        for new_plugin in event.new_plugins.drain(..) {
            match new_plugin.status {
                // This means the plugin successfully activated and returned
                // its new audio/event port configuration and its new
                // parameter configuration.
                PluginActivationStatus::Activated { new_handle, new_param_values } => {
                    // There is only ever one sample browser plugin.
                    if engine_handles.sample_browser_plug_handle.is_none() {
                        if new_plugin.plugin_id.rdn() == SAMPLE_BROWSER_PLUG_RDN {
                            engine_handles.sample_browser_plug_handle = Some(new_handle);
                            // TODO: Update state of the gain parameter for this plugin.
                        }
                    }

                    // TODO: Handle other plugins.
                }
                // This means that the plugin loaded but did not activate yet. This
                // can happen when the user loads a project with a deactivated
                // plugin.
                PluginActivationStatus::Inactive => {
                    // TODO: Deactivate plugin in UI.
                }
                // There was an error loading the plugin.
                PluginActivationStatus::LoadError(e) => {
                    // TODO: Display error to user in UI.
                }
                // There was an error activating the plugin.
                PluginActivationStatus::ActivationError(e) => {
                    // TODO: Display error to user in UI.
                }
            }
        }

        // TODO
    }

    /// Sent whenever a plugin becomes activated after being deactivated or
    /// when the plugin restarts.
    ///
    /// Make sure your UI updates the port configuration on this plugin.
    fn on_plugin_activated(
        &mut self,
        plugin_id: PluginInstanceID,
        new_handle: PluginHandle,
        new_param_values: FnvHashMap<ParamID, f64>,
    ) {
        // TODO
    }

    /// Sent whenever a plugin becomes deactivated. When a plugin is deactivated
    /// you cannot access any of its methods until it is reactivated.
    fn on_plugin_deactivated(
        &mut self,
        plugin_id: PluginInstanceID,
        // If this is `Ok(())`, then it means the plugin was gracefully
        // deactivated from user request.
        //
        // If this is `Err(e)`, then it means the plugin became deactivated
        // because it failed to restart.
        status: Result<(), ActivatePluginError>,
    ) {
        // TODO
    }

    fn on_plugin_params_modified(
        &mut self,
        plugin_id: PluginInstanceID,
        modified_params: SmallVec<[ParamModifiedInfo; 4]>,
    ) {
        // TODO
    }

    /// A new CLAP plugin scan path was added.
    fn on_clap_scan_path_added(&mut self, path: PathBuf) {
        // TODO
    }

    /// A CLAP plugin scan path was removed.
    fn on_clap_scan_path_removed(&mut self, path: PathBuf) {
        // TODO
    }

    /// A request to rescan all plugin directories has finished. Update
    /// the list of available plugins in your UI.
    fn on_plugin_scanner_rescan_finished(&mut self, info: RescanPluginDirectoriesRes) {
        // TODO
    }
}

impl Model for UiState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|channel_event, _| match channel_event {
            // Select a single channel
            ChannelEvent::SelectChannel(index) => {
                deselect_channels(&mut self.channels);

                if let Some(channel_data) = self.channels.get_mut(*index) {
                    channel_data.selected = true;
                }
            }

            // Select a channel and any children in the same group
            ChannelEvent::SelectChannelGroup(index) => {
                println!("Select channel group: {}", index);
                deselect_channels(&mut self.channels);

                let mut selected = vec![];

                select_channel(&self.channels, *index, &mut selected);

                for idx in selected.iter() {
                    if let Some(channel_data) = self.channels.get_mut(*idx) {
                        channel_data.selected = true;
                    }
                }
            }

            // Add a new channel to the channels panel
            ChannelEvent::AddChannel => {
                deselect_channels(&mut self.channels);

                let channel_id = self.channels.len();

                // Create a new channel
                self.channels.push(ChannelState {
                    name: String::from("New Channel"),
                    path: PathBuf::from("New Channel"),
                    color: ChannelBaseColor::Color(Color::rgb(200, 50, 50)),
                    selected: true,
                    ..Default::default()
                });

                // Add new channel to master group
                if let Some(master) = self.channels.get_mut(0) {
                    master.subchannels.push(channel_id);
                }
            }

            // Remove the specified channel from the channels panel
            ChannelEvent::RemoveChannel => {}
        });

        self.panels.event(cx, event);
        self.timeline_grid.event(cx, event);
        self.browser.event(cx, event);
    }
}

// Helper function for recursively collecting the indices of selected channels
fn select_channel(channel_data: &Vec<ChannelState>, index: usize, selected: &mut Vec<usize>) {
    if let Some(data) = channel_data.get(index) {
        selected.push(index);
        for subchannel in data.subchannels.iter() {
            select_channel(channel_data, *subchannel, selected);
        }
    }
}

// Helper function for deselecting all channels
fn deselect_channels(channel_data: &mut Vec<ChannelState>) {
    for channel in channel_data.iter_mut() {
        channel.selected = false;
    }
}
