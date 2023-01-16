// BIG TODO: Have the entire engine run in a separate process for
// crash protection from buggy plugins.

use std::time::{Duration, Instant};

use meadowlark_engine::engine::error::EngineCrashError;
use meadowlark_engine::{
    engine::{
        modify_request::{ConnectEdgeReq, EdgeReqPortID, ModifyGraphRequest, PluginIDReq},
        ActivateEngineSettings, ActivatedEngineInfo, EngineMainThread, EngineSettings,
        PluginStatus,
    },
    graph::PortType,
    plugin_host::PluginHostSaveState,
};
use meadowlark_plugin_api::transport::LoopState;
use meadowlark_plugin_api::{HostInfo, ParamID, PluginInstanceID};

use crate::resource::ResourceLoader;
use crate::state_system::time::{FrameTime, TempoMap};
use crate::state_system::SourceState;

use crate::plugins::sample_browser_plug::{
    SampleBrowserPlugFactory, SampleBrowserPlugHandle, SAMPLE_BROWSER_PLUG_RDN,
};
use crate::plugins::timeline_track_plug::{
    TimelineTrackPlugFactory, TimelineTrackPlugHandle, TIMELINE_TRACK_PLUG_RDN,
};

pub mod system_io;

use system_io::SystemIOStreamHandle;

// TODO: Have these be configurable.
const MIN_FRAMES: u32 = 1;
const MAX_FRAMES: u32 = 512;
const GRAPH_IN_CHANNELS: u16 = 2;
const GRAPH_OUT_CHANNELS: u16 = 2;

pub static GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(3);

pub struct EngineHandle {
    pub ds_engine: EngineMainThread,
    pub activated_handles: Option<ActivatedEngineHandles>,

    pub next_timer_instant: Instant,
    pub next_garbage_collect_instant: Instant,

    pub system_io_stream_handle: SystemIOStreamHandle,
}

impl EngineHandle {
    pub fn new(state: &SourceState) -> Self {
        // TODO: Use rainout instead of cpal once it's ready.
        // TODO: Load settings from a save file rather than spawning
        // a stream with default settings.
        let mut system_io_stream_handle =
            crate::engine_handle::system_io::temp_spawn_cpal_default_output_only().unwrap();

        let (mut ds_engine, first_timer_instant, internal_plugins_scan_res) = EngineMainThread::new(
            HostInfo::new(
                "Meadowlark".into(),                   // host name
                env!("CARGO_PKG_VERSION").into(),      // host version
                Some("Meadowlark".into()),             // vendor
                Some("https://meadowlark.app".into()), // url
            ),
            EngineSettings::default(),
            vec![Box::new(SampleBrowserPlugFactory), Box::new(TimelineTrackPlugFactory)], // list of internal plugins
        );

        log::info!("{:?}", &internal_plugins_scan_res);

        let (seek_to_frame, loop_state, tempo_map) = if let Some(project_state) = &state.project {
            let seek_to_frame = project_state
                .tempo_map
                .timestamp_to_nearest_frame_round(project_state.playhead_last_seeked);

            let loop_state = if project_state.loop_active {
                LoopState::Active {
                    loop_start_frame: project_state
                        .tempo_map
                        .timestamp_to_nearest_frame_round(project_state.loop_start)
                        .0,
                    loop_end_frame: project_state
                        .tempo_map
                        .timestamp_to_nearest_frame_round(project_state.loop_end)
                        .0,
                }
            } else {
                LoopState::Inactive
            };

            (seek_to_frame, loop_state, Box::new(project_state.tempo_map.clone()))
        } else {
            (FrameTime(0), LoopState::Inactive, Box::new(TempoMap::default()))
        };

        let (engine_info, ds_engine_audio_thread) = ds_engine
            .activate_engine(
                seek_to_frame.0,
                loop_state,
                tempo_map,
                ActivateEngineSettings {
                    sample_rate: system_io_stream_handle.sample_rate(),
                    min_frames: MIN_FRAMES,
                    max_frames: MAX_FRAMES,
                    num_audio_in_channels: GRAPH_IN_CHANNELS,
                    num_audio_out_channels: GRAPH_OUT_CHANNELS,
                    hard_clip_outputs: true,
                    ..Default::default()
                },
            )
            .unwrap();

        system_io_stream_handle.on_engine_activated(ds_engine_audio_thread);

        let mut sample_browser_plug_key = None;
        let mut timeline_track_plug_key = None;
        for res in internal_plugins_scan_res.iter() {
            if let Ok(res) = res {
                if res.rdn == SAMPLE_BROWSER_PLUG_RDN {
                    sample_browser_plug_key = Some(res.clone());
                } else if res.rdn == TIMELINE_TRACK_PLUG_RDN {
                    timeline_track_plug_key = Some(res.clone());
                }
            }
        }
        let sample_browser_plug_key = sample_browser_plug_key.unwrap();
        let timeline_track_plug_key = timeline_track_plug_key.unwrap();

        let graph_out_id = engine_info.graph_out_id.clone();

        // Add a sample browser plugin to the graph, and connect it directly
        // to the graph output.
        let mut res = ds_engine
            .modify_graph(ModifyGraphRequest {
                add_plugin_instances: vec![PluginHostSaveState::new_with_default_state(
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
                        dst_plugin_id: PluginIDReq::Existing(graph_out_id.clone()),
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
                f64::from(state.app.browser_panel.volume_normalized),
            )
            .unwrap();

        let mut resource_loader = ResourceLoader::new(system_io_stream_handle.sample_rate());

        let mut timeline_track_plug_handles: Vec<TimelineTrackPlugHandle> = Vec::new();
        if let Some(project_state) = &state.project {
            for track_state in project_state.tracks.iter() {
                // Create a timeline track plugin and add it to the graph.

                // TODO: Tracks that don't have stereo outputs.
                let mut res = ds_engine
                    .modify_graph(ModifyGraphRequest {
                        add_plugin_instances: vec![PluginHostSaveState::new_with_default_state(
                            timeline_track_plug_key.clone(),
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
                                dst_plugin_id: PluginIDReq::Existing(graph_out_id.clone()),
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

                let timeline_track_plug_res = res.new_plugins.remove(0);
                let timeline_track_plug_id = timeline_track_plug_res.plugin_id;
                let timeline_track_plug_host =
                    ds_engine.plugin_host_mut(&timeline_track_plug_id).unwrap();
                let mut timeline_track_plug_handle =
                    if let PluginStatus::Activated(status) = timeline_track_plug_res.status {
                        *(status
                            .internal_handle
                            .unwrap()
                            .downcast::<TimelineTrackPlugHandle>()
                            .unwrap())
                    } else {
                        panic!("Timeline track plugin failed to activate");
                    };

                // Fill the timeline track plugin with the corresponding state.
                timeline_track_plug_handle.sync_from_track_state(
                    track_state,
                    &project_state.tempo_map,
                    &mut resource_loader,
                );

                timeline_track_plug_handles.push(timeline_track_plug_handle);
            }
        }

        let activated_handles = ActivatedEngineHandles {
            engine_info,
            sample_browser_plug_id,
            sample_browser_plug_params,
            sample_browser_plug_handle,
            timeline_track_plug_handles,
            resource_loader,
        };

        Self {
            ds_engine,
            activated_handles: Some(activated_handles),
            next_timer_instant: first_timer_instant,
            next_garbage_collect_instant: Instant::now() + GARBAGE_COLLECT_INTERVAL,
            system_io_stream_handle,
        }
    }
}

pub enum EnginePollStatus {
    Ok,
    EngineDeactivatedGracefully,
    EngineCrashed(Box<EngineCrashError>),
}

pub struct ActivatedEngineHandles {
    pub engine_info: ActivatedEngineInfo,
    pub resource_loader: ResourceLoader,

    pub sample_browser_plug_id: PluginInstanceID,
    pub sample_browser_plug_params: Vec<ParamID>,
    pub sample_browser_plug_handle: SampleBrowserPlugHandle,

    pub timeline_track_plug_handles: Vec<TimelineTrackPlugHandle>,
}
