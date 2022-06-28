//! # Program (State) Layer
//!
//! This layer owns the state of the program.
//!
//! It is solely in charge of mutating this state. The backend layer and the UI
//! layer cannot mutate this state directly (with the exception of some
//! UI-specific state that does not need to be undo-able such as panel or window
//! size). The backend layer indirectly mutates this state by sending events to
//! the program layer, and the ui layer indirectly mutates this state by calling
//! methods on the ProgramState struct which the UI layer owns.
//!
//! The program layer also owns the handle to the audio thread and is in charge
//! of connecting to the system's audio and MIDI devices. It is also in charge
//! of some offline DSP such as resampling audio clips.

pub mod program_state;
pub mod system_io;

use dropseed::plugin::PluginSaveState;
use dropseed::plugins::sample_browser::{
    SampleBrowserPlugFactory, SampleBrowserPlugHandle, SAMPLE_BROWSER_PLUG_RDN,
};
use dropseed::{
    transport::TransportHandle, ActivateEngineSettings, ActivatePluginError, DSEngineEvent,
    DSEngineHandle, DSEngineRequest, EdgeReq, EdgeReqPortID, EngineActivatedInfo,
    EngineDeactivatedInfo, HostInfo, ModifyGraphRequest, ModifyGraphRes, ParamID,
    ParamModifiedInfo, PluginActivationStatus, PluginEvent, PluginHandle, PluginIDReq,
    PluginInstanceID, PluginScannerEvent, PortType, RescanPluginDirectoriesRes,
};
use fnv::FnvHashMap;
use meadowlark_core_types::{MusicalTime, SampleRate};
use smallvec::SmallVec;
use std::{fmt::Debug, path::PathBuf};
use vizia::prelude::*;

pub use program_state::ProgramState;

use program_state::{
    ChannelRackOrientation, ChannelState, LaneState, LaneStates, PanelState, PatternState,
    TimelineGridState,
};

use self::{program_state::BrowserState, system_io::SystemIOStreamHandle};

// TODO: Have these be configurable.
const MIN_FRAMES: u32 = 1;
const MAX_FRAMES: u32 = 512;
const GRAPH_IN_CHANNELS: u16 = 2;
const GRAPH_OUT_CHANNELS: u16 = 2;

struct EngineHandles {
    ds_handle: DSEngineHandle,

    activated_handles: Option<ActivatedEngineHandles>,
    sample_browser_plug_handle: Option<PluginHandle>,
}

struct ActivatedEngineHandles {
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

/// This is in charge of keeping track of state for the whole program.
///
/// The UI must continually call `ProgramLayer::poll()` (on every frame or an
/// equivalent timer).
#[derive(Lens)]
pub struct ProgramLayer {
    /// The state of the whole program.
    ///
    /// Unless explicitely stated, the UI may NOT directly mutate the state of any
    /// of these variables. It is intended for the UI to call the methods on this
    /// struct in order to mutate state.
    pub state: ProgramState,

    #[lens(ignore)]
    system_io_stream_handle: Option<SystemIOStreamHandle>,

    #[lens(ignore)]
    engine_handles: Option<(EngineHandles, crossbeam::channel::Receiver<DSEngineEvent>)>,
}

impl Debug for ProgramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("ProgramLayer");
        f.field("state", &self.state);
        f.finish()
    }
}

impl ProgramLayer {
    // Create some dummy state for now
    pub fn new() -> Result<Self, ()> {
        // This is temporary. Eventually we will have a more sophisticated and
        // configurable system using `rainout`.
        let system_io_stream_handle = system_io::temp_spawn_cpal_default_output_only().unwrap();

        Ok(ProgramLayer {
            state: ProgramState {
                engine_running: false,
                notification_log: Vec::new(),
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

                patterns: vec![PatternState { name: String::from("Drum Group 1"), channel: 1 }],
                timeline_grid: TimelineGridState {
                    horizontal_zoom_level: 1.0,
                    vertical_zoom_level: 1.0,
                    left_start: MusicalTime::from_beats(0),
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
                    project_length: MusicalTime::from_beats(16),
                    used_lanes: 0,
                },
                browser: BrowserState::default(),
                panels: PanelState {
                    channel_rack_orientation: ChannelRackOrientation::Horizontal,
                    hide_patterns: false,
                    hide_piano_roll: false,
                    browser_width: 100.0,
                    show_browser: true,
                },
            },
            system_io_stream_handle: Some(system_io_stream_handle),
            engine_handles: None,
        })
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
                vec![Box::new(SampleBrowserPlugFactory)],
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
                    activated_handles: None,
                    sample_browser_plug_handle: None,
                },
                engine_rx,
            ));
        } else {
            log::warn!("Cannot activate engine until a system IO stream is started");
        }
    }

    pub fn sample_browser_play_sample(&mut self, path: &PathBuf) {
        // TODO
    }

    pub fn sample_browser_replay(&mut self) {
        // TODO
    }

    pub fn sample_browser_stop_playing(&mut self) {
        // TODO
    }

    /// TODO
    pub fn poll(&mut self) {
        if let Some((engine_handles, engine_rx)) = &mut self.engine_handles {
            //let EngineHandles { handle, rx, activated_handles, sample_browser_plug_handle } = engine_handle;

            for msg in engine_rx.try_iter() {
                match msg {
                    // TODO: Hint to the compiler that this is by far the most likely event?
                    DSEngineEvent::Plugin(PluginEvent::ParamsModified {
                        plugin_id,
                        modified_params,
                    }) => {
                        Self::on_plugin_params_modified(
                            plugin_id,
                            modified_params,
                            &mut self.state,
                            engine_handles,
                        );
                    }
                    // TODO: Hint to the compiler that this is the next most likely event?
                    DSEngineEvent::AudioGraphModified(event) => {
                        Self::on_audio_graph_modified(event, &mut self.state, engine_handles);
                    }
                    DSEngineEvent::Plugin(PluginEvent::Activated {
                        plugin_id,
                        new_handle,
                        new_param_values,
                    }) => {
                        Self::on_plugin_activated(
                            plugin_id,
                            new_handle,
                            new_param_values,
                            &mut self.state,
                            engine_handles,
                        );
                    }
                    DSEngineEvent::Plugin(PluginEvent::Deactivated { plugin_id, status }) => {
                        Self::on_plugin_deactivated(
                            plugin_id,
                            status,
                            &mut self.state,
                            engine_handles,
                        );
                    }
                    DSEngineEvent::EngineDeactivated(event) => {
                        Self::on_engine_deactivated(event, &mut self.state, engine_handles);
                    }
                    DSEngineEvent::EngineActivated(event) => {
                        Self::on_engine_activated(event, &mut self.state, engine_handles);
                    }
                    DSEngineEvent::AudioGraphCleared => {
                        Self::on_audio_graph_cleared(&mut self.state, engine_handles);
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::ClapScanPathAdded(path)) => {
                        Self::on_clap_scan_path_added(path, &mut self.state, engine_handles);
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::ClapScanPathRemoved(path)) => {
                        Self::on_clap_scan_path_removed(path, &mut self.state, engine_handles);
                    }
                    DSEngineEvent::PluginScanner(PluginScannerEvent::RescanFinished(event)) => {
                        Self::on_plugin_scanner_rescan_finished(
                            event,
                            &mut self.state,
                            engine_handles,
                        );
                    }
                    unkown_event => {
                        log::warn!("{:?}", unkown_event);
                    }
                }
            }
        }
    }

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
        event: EngineDeactivatedInfo,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        engine_handles.activated_handles = None;
        engine_handles.sample_browser_plug_handle = None;

        // TODO
    }

    /// This message is sent whenever the engine successfully activates.
    fn on_engine_activated(
        event: EngineActivatedInfo,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // Collect the keys for the internal plugins.
        let mut sample_browser_plug_key = None;
        for p in engine_handles.ds_handle.internal_plugins_res.iter() {
            if let Ok(key) = p {
                if &key.rdn == SAMPLE_BROWSER_PLUG_RDN {
                    sample_browser_plug_key = Some(key.clone());
                    break;
                }
            }
        }
        let sample_browser_plug_key = sample_browser_plug_key.unwrap();

        // Add the sample-browser plugin and connect it directly to the output.
        engine_handles.ds_handle.send(DSEngineRequest::ModifyGraph(ModifyGraphRequest {
            add_plugin_instances: vec![PluginSaveState::new_with_default_preset(
                sample_browser_plug_key,
            )],
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
            ],
            disconnect_edges: vec![],
        }));

        // TODO
    }

    /// When this message is received, it means that the audio graph is starting
    /// the process of restoring from a save state.
    ///
    /// Reset your UI as if you are loading up a project for the first time, and
    /// wait for the `AudioGraphModified` event to repopulate the UI.
    ///
    /// If the audio graph is in an invalid state as a result of restoring from
    /// the save state, then the `EngineDeactivated` event will be sent instead.
    fn on_audio_graph_cleared(state: &mut ProgramState, engine_handles: &mut EngineHandles) {
        // TODO
    }

    /// This message is sent whenever the audio graph has been modified.
    ///
    /// Be sure to update your UI from this new state.
    fn on_audio_graph_modified(
        mut event: ModifyGraphRes,
        state: &mut ProgramState,
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
        plugin_id: PluginInstanceID,
        new_handle: PluginHandle,
        new_param_values: FnvHashMap<ParamID, f64>,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }

    /// Sent whenever a plugin becomes deactivated. When a plugin is deactivated
    /// you cannot access any of its methods until it is reactivated.
    fn on_plugin_deactivated(
        plugin_id: PluginInstanceID,
        // If this is `Ok(())`, then it means the plugin was gracefully
        // deactivated from user request.
        //
        // If this is `Err(e)`, then it means the plugin became deactivated
        // because it failed to restart.
        status: Result<(), ActivatePluginError>,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }

    fn on_plugin_params_modified(
        plugin_id: PluginInstanceID,
        modified_params: SmallVec<[ParamModifiedInfo; 4]>,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }

    /// A new CLAP plugin scan path was added.
    fn on_clap_scan_path_added(
        path: PathBuf,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }

    /// A CLAP plugin scan path was removed.
    fn on_clap_scan_path_removed(
        path: PathBuf,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }

    /// A request to rescan all plugin directories has finished. Update
    /// the list of available plugins in your UI.
    fn on_plugin_scanner_rescan_finished(
        info: RescanPluginDirectoriesRes,
        state: &mut ProgramState,
        engine_handles: &mut EngineHandles,
    ) {
        // TODO
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProgramEvent {
    // ----- General -----

    // Project
    SaveProject,
    LoadProject,

    // ----- Timeline -----

    // Insertion
    InsertLane,
    DuplicateSelectedLanes,

    // Selection
    SelectLane(usize),
    SelectLaneAbove,
    SelectLaneBelow,
    SelectAllLanes,
    MoveSelectedLanesUp,
    MoveSelectedLanesDown,

    // Deletion
    DeleteSelectedLanes,
    ToggleLaneActivation,

    // Zoom
    ZoomInVertically,
    ZoomOutVertically,

    // Height
    IncreaseSelectedLaneHeight,
    DecreaseSelectedLaneHeight,

    // Activation
    ActivateSelectedLanes,
    DeactivateSelectedLanes,
    ToggleSelectedLaneActivation,
}

impl Model for ProgramLayer {
    // Update the program layer here
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|program_event, _| match program_event {
            ProgramEvent::SaveProject => {
                let save_state = serde_json::to_string(&self.state).unwrap();
                std::fs::write("project.json", save_state).unwrap();
            }
            ProgramEvent::LoadProject => {
                let save_state = std::fs::read_to_string("project.json").unwrap();
                let project_state = serde_json::from_str(&save_state).unwrap();
                self.state = project_state;
            }
            _ => {}
        });

        self.state.event(cx, event);
    }
}
