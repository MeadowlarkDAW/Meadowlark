use basedrop::{Collector, Shared, SharedCell};
use fnv::FnvHashMap;

pub mod nodes;

use crate::shared_state::{PortType, SharedState, SharedStateManager};

pub struct FrontendState {
    pub stereo_sine_gen_handles: FnvHashMap<String, nodes::sine_gen::StereoSineGenNodeHandle>,
    pub mono_gain_handles: FnvHashMap<String, nodes::gain::MonoGainNodeHandle>,
    pub stereo_gain_handles: FnvHashMap<String, nodes::gain::StereoGainNodeHandle>,
    pub mono_monitor_handles: FnvHashMap<String, nodes::monitor::MonoMonitorNodeHandle>,
    pub stereo_monitor_handles: FnvHashMap<String, nodes::monitor::StereoMonitorNodeHandle>,

    shared_state: SharedStateManager,
}

impl FrontendState {
    pub fn new(
        max_audio_frames: usize,
        sample_rate: f32,
    ) -> (Self, Shared<SharedCell<SharedState>>) {
        let collector = Collector::new();

        let (shared_state, rt_shared_state) =
            SharedStateManager::new(max_audio_frames, sample_rate);

        let mut new_self = Self {
            shared_state,
            stereo_sine_gen_handles: FnvHashMap::default(),
            mono_gain_handles: FnvHashMap::default(),
            stereo_gain_handles: FnvHashMap::default(),
            mono_monitor_handles: FnvHashMap::default(),
            stereo_monitor_handles: FnvHashMap::default(),
        };

        new_self.test_setup();

        (new_self, rt_shared_state)
    }

    /// A temporary test setup: "sine wave generator" -> "gain knob" -> "db meter".
    pub fn test_setup(&mut self) {
        let sine_gen_id = String::from("sine_gen");
        let (sine_gen_node, sine_gen_node_handle) =
            nodes::sine_gen::StereoSineGenNode::new(440.0, 1.0, &self.shared_state.coll_handle());

        let gain_id = String::from("gain");
        let (gain_node, gain_node_handle) =
            nodes::gain::StereoGainNode::new(1.0, &self.shared_state.coll_handle());

        let monitor_id = String::from("monitor");
        let (monitor_node, monitor_node_handle) =
            nodes::monitor::StereoMonitorNode::new(2048, true, &self.shared_state.coll_handle());

        self.shared_state
            .modify_graph(|mut graph_state, _coll_handle| {
                graph_state
                    .add_new_node(&sine_gen_id, Box::new(sine_gen_node))
                    .unwrap();
                graph_state
                    .add_new_node(&gain_id, Box::new(gain_node))
                    .unwrap();
                graph_state
                    .add_new_node(&monitor_id, Box::new(monitor_node))
                    .unwrap();

                graph_state
                    .add_port_connection(PortType::Audio, &sine_gen_id, 0, &gain_id, 0)
                    .unwrap();
                graph_state
                    .add_port_connection(PortType::Audio, &sine_gen_id, 1, &gain_id, 1)
                    .unwrap();

                graph_state
                    .add_port_connection(PortType::Audio, &gain_id, 0, &monitor_id, 0)
                    .unwrap();
                graph_state
                    .add_port_connection(PortType::Audio, &gain_id, 1, &monitor_id, 1)
                    .unwrap();
            });

        self.stereo_sine_gen_handles
            .insert(sine_gen_id, sine_gen_node_handle);
        self.stereo_gain_handles.insert(gain_id, gain_node_handle);
        self.stereo_monitor_handles
            .insert(monitor_id, monitor_node_handle);
    }

    /// Call periodically to collect garbage in the rt thread.
    ///
    /// TODO: Actually do this somewhere!
    pub fn collect(&mut self) {
        self.shared_state.collect();
    }
}
