use basedrop::Shared;
use meadowlark_plugin_api::ProcInfo;
use std::fmt::Write;

pub(crate) mod tasks;

pub use tasks::TransportHandle;

use crate::{graph::shared_pools::SharedTransportTask, plugin_host::PluginHostProcessorWrapper};

use tasks::{GraphInTask, GraphOutTask, Task};

pub struct ProcessorSchedule {
    tasks: Vec<Task>,

    graph_in_task: GraphInTask,
    graph_out_task: GraphOutTask,
    transport_task: SharedTransportTask,

    /// For the plugins that are queued to be removed, make sure that
    /// the plugin's processor part is dropped in the process thread.
    plugin_processors_to_stop: Vec<Shared<PluginHostProcessorWrapper>>,

    max_block_size: usize,

    version: u64,
}

impl ProcessorSchedule {
    pub(crate) fn new(
        tasks: Vec<Task>,
        graph_in_task: GraphInTask,
        graph_out_task: GraphOutTask,
        transport_task: SharedTransportTask,
        // For the plugins that are queued to be removed, make sure that
        // the plugin's processor part is dropped in the process thread.
        plugin_processors_to_stop: Vec<Shared<PluginHostProcessorWrapper>>,
        max_block_size: usize,
        version: u64,
    ) -> Self {
        Self {
            tasks,
            graph_in_task,
            graph_out_task,
            transport_task,
            plugin_processors_to_stop,
            max_block_size,
            version,
        }
    }

    pub(crate) fn new_empty(
        max_block_size: usize,
        transport_task: SharedTransportTask,
        plugin_processors_to_stop: Vec<Shared<PluginHostProcessorWrapper>>,
        version: u64,
    ) -> Self {
        Self {
            tasks: Vec::new(),
            graph_in_task: GraphInTask::default(),
            graph_out_task: GraphOutTask::default(),
            transport_task,
            plugin_processors_to_stop,
            max_block_size,
            version,
        }
    }

    pub(crate) fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

impl std::fmt::Debug for ProcessorSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let _ = writeln!(f, "ProcessorSchedule | version: {} {{", &self.version);

        if !self.graph_in_task.audio_in.is_empty() {
            let mut s2 = String::new();
            for b in self.graph_in_task.audio_in.iter() {
                let _ = write!(s2, "{:?}, ", b.id());
            }

            let _ = writeln!(s, "    graph_audio_in: {},", s2);
        }

        for t in self.tasks.iter() {
            let _ = writeln!(s, "    {:?},", t);
        }

        if !self.graph_out_task.audio_out.is_empty() {
            let mut s2 = String::new();
            for b in self.graph_out_task.audio_out.iter() {
                let _ = write!(s2, "{:?}, ", b.id());
            }

            let _ = writeln!(s, "    graph_audio_out: {}", s2);
        }

        write!(f, "{}", s)
    }
}

impl ProcessorSchedule {
    pub fn process_interleaved(&mut self, audio_in: &[f32], audio_out: &mut [f32]) {
        // For the plugins that are queued to be removed, make sure that
        // their processors are dropped on the process thread.
        for plugin_proc in self.plugin_processors_to_stop.drain(..) {
            let mut plugin_proc = plugin_proc.borrow_mut();
            *plugin_proc = None;
        }

        let audio_in_channels = self.graph_in_task.audio_in.len();
        let audio_out_channels = self.graph_out_task.audio_out.len();

        if audio_in_channels != 0 && audio_out_channels != 0 {
            assert_eq!(audio_in.len() / audio_in_channels, audio_out.len() / audio_out_channels);
        }

        let total_frames = if audio_in_channels > 0 {
            let total_frames = audio_in.len() / audio_in_channels;

            assert_eq!(audio_out.len(), audio_out_channels * total_frames);

            total_frames
        } else if audio_out_channels > 0 {
            audio_out.len() / audio_out_channels
        } else {
            return;
        };

        if total_frames == 0 {
            return;
        }

        let mut processed_frames = 0;
        while processed_frames < total_frames {
            let frames = (total_frames - processed_frames).min(self.max_block_size);

            // De-interlace the audio in stream to the graph input buffers.
            for (channel_i, buffer) in self.graph_in_task.audio_in.iter().enumerate() {
                let buffer = &mut buffer.borrow_mut().data[0..frames];

                // TODO: Check that the compiler is properly eliding bounds checking.
                for i in 0..frames {
                    buffer[i] = audio_in[((i + processed_frames) * audio_in_channels) + channel_i];
                }
            }

            let transport = self.transport_task.borrow_mut().process(frames);

            let proc_info = ProcInfo {
                steady_time: -1, // TODO
                frames,
                transport,
                schedule_version: self.version,
            };

            for task in self.tasks.iter_mut() {
                task.process(&proc_info)
            }

            // Interlace the graph output buffers to the audio out stream.
            for (channel_i, buffer) in self.graph_out_task.audio_out.iter().enumerate() {
                let buffer = &buffer.borrow().data[0..frames];

                // TODO: Check that the compiler is properly eliding bounds checking.
                for i in 0..frames {
                    audio_out[((i + processed_frames) * audio_out_channels) + channel_i] =
                        buffer[i];
                }
            }

            processed_frames += frames;
        }
    }

    pub fn deactivate(&mut self) {
        // Make sure we drop all plugin processors in the process thread
        // when deactivating the engine.
        for plugin_proc in self.plugin_processors_to_stop.drain(..) {
            let mut plugin_proc = plugin_proc.borrow_mut();
            *plugin_proc = None;
        }
    }
}
