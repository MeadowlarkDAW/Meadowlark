use meadowlark_plugin_api::ProcInfo;
use std::fmt::{Debug, Error, Formatter, Write};

mod delay_comp_task;
mod graph_in_out_task;
mod plugin_task;
mod sum_task;
mod transport_task;
mod unloaded_plugin_task;

pub use transport_task::TransportHandle;

pub(crate) use delay_comp_task::{
    AudioDelayCompNode, AudioDelayCompTask, AutomationDelayCompNode, AutomationDelayCompTask,
    NoteDelayCompNode, NoteDelayCompTask, SharedAudioDelayCompNode, SharedAutomationDelayCompNode,
    SharedNoteDelayCompNode,
};
pub(crate) use graph_in_out_task::{GraphInTask, GraphOutTask};
pub(crate) use plugin_task::PluginTask;
pub(crate) use sum_task::{AudioSumTask, AutomationSumTask, NoteSumTask};
pub(crate) use transport_task::TransportTask;
pub(crate) use unloaded_plugin_task::UnloadedPluginTask;

pub(crate) enum Task {
    Plugin(PluginTask),
    AudioSum(AudioSumTask),
    NoteSum(NoteSumTask),
    AutomationSum(AutomationSumTask),
    AudioDelayComp(AudioDelayCompTask),
    NoteDelayComp(NoteDelayCompTask),
    AutomationDelayComp(AutomationDelayCompTask),
    UnloadedPlugin(UnloadedPluginTask),
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // TODO: Move the debug printing for enum variants into the respective modules.
        match self {
            Task::Plugin(t) => {
                let mut f = f.debug_struct("Plugin");

                f.field("id", &t.plugin_id);

                if !t.buffers.audio_in.is_empty() {
                    let mut s = String::new();
                    for b in t.buffers.audio_in.iter() {
                        let _ = write!(s, "{:?}, ", b);
                    }

                    f.field("audio_in", &s);
                }

                if !t.buffers.audio_out.is_empty() {
                    let mut s = String::new();
                    for b in t.buffers.audio_out.iter() {
                        let _ = write!(s, "{:?}, ", b);
                    }

                    f.field("audio_out", &s);
                }

                if let Some((automation_in_buffer, do_clear)) =
                    &t.event_buffers.automation_in_buffer
                {
                    f.field(
                        "automation_in",
                        &format!("{:?} clear: {}", &automation_in_buffer.id(), do_clear),
                    );
                }

                if let Some(automation_out_buffer) = &t.event_buffers.automation_out_buffer {
                    f.field("automation_out", &format!("{:?}", automation_out_buffer.id()));
                }

                if !t.event_buffers.note_in_buffers.is_empty() {
                    let mut s = String::new();
                    for buffer in t.event_buffers.note_in_buffers.iter() {
                        let _ = write!(s, "{:?}, ", buffer.id());
                    }

                    f.field("note_in", &s);
                }

                if !t.event_buffers.note_out_buffers.is_empty() {
                    let mut s = String::new();
                    for buffer in t.event_buffers.note_out_buffers.iter() {
                        let _ = write!(s, "{:?}, ", buffer.id());
                    }

                    f.field("note_out", &s);
                }

                if !t.clear_audio_in_buffers.is_empty() {
                    let mut s = String::new();
                    for b in t.clear_audio_in_buffers.iter() {
                        let _ = write!(s, "{:?}, ", &b.id());
                    }

                    f.field("clear_audio_in", &s);
                }

                if !t.event_buffers.clear_note_in_buffers.is_empty() {
                    let mut s = String::new();
                    for b in t.event_buffers.clear_note_in_buffers.iter() {
                        let _ = write!(s, "{:?}, ", &b.id());
                    }

                    f.field("clear_note_in", &s);
                }

                f.finish()
            }
            Task::AudioSum(t) => {
                let mut f = f.debug_struct("AudioSum");

                let mut s = String::new();
                for b in t.audio_in.iter() {
                    let _ = write!(s, "{:?}, ", b.id());
                }
                f.field("audio_in", &s);

                f.field("audio_out", &format!("{:?}", t.audio_out.id()));

                f.finish()
            }
            Task::NoteSum(t) => {
                let mut f = f.debug_struct("NoteSum");

                let mut s = String::new();
                for b in t.note_in.iter() {
                    let _ = write!(s, "{:?}, ", b.id());
                }
                f.field("note_in", &s);

                f.field("note_out", &format!("{:?}", t.note_out.id()));

                f.finish()
            }
            Task::AutomationSum(t) => {
                let mut f = f.debug_struct("AutomationSum");

                let mut s = String::new();
                for b in t.input.iter() {
                    let _ = write!(s, "{:?}, ", b.id());
                }
                f.field("input", &s);

                f.field("output", &format!("{:?}", t.output.id()));

                f.finish()
            }
            Task::AudioDelayComp(t) => {
                let mut f = f.debug_struct("AudioDelayComp");

                f.field("audio_in", &t.audio_in.id());
                f.field("audio_out", &t.audio_out.id());
                f.field("delay", &t.shared_node.delay);

                f.finish()
            }
            Task::NoteDelayComp(t) => {
                let mut f = f.debug_struct("NoteDelayComp");

                f.field("note_in", &t.note_in.id());
                f.field("note_out", &t.note_out.id());
                f.field("delay", &t.shared_node.delay);

                f.finish()
            }
            Task::AutomationDelayComp(t) => {
                let mut f = f.debug_struct("AutomationDelayComp");

                f.field("input", &t.input.id());
                f.field("output", &t.output.id());
                f.field("delay", &t.shared_node.delay);

                f.finish()
            }
            Task::UnloadedPlugin(t) => {
                let mut f = f.debug_struct("UnloadedPlugin");

                let mut s = String::new();
                for (b_in, b_out) in t.audio_through.iter() {
                    let _ = write!(s, "(in: {:?}, out: {:?})", b_in.id(), b_out.id());
                }
                f.field("audio_through", &s);

                let mut s = String::new();
                for b in t.clear_audio_out.iter() {
                    let _ = write!(s, "{:?}, ", b.id());
                }
                f.field("clear_audio_out", &s);

                if let Some(automation_out_buffer) = &t.clear_automation_out {
                    f.field("clear_automation_out", &format!("{:?}", automation_out_buffer.id()));
                }

                if !t.clear_note_out.is_empty() {
                    let mut has_buffer = false;
                    let mut s = String::new();
                    for buffer in t.clear_note_out.iter() {
                        has_buffer = true;
                        let _ = write!(s, "{:?}, ", buffer.id());
                    }

                    if has_buffer {
                        f.field("clear_note_out", &s);
                    }
                }

                f.finish()
            }
        }
    }
}

impl Task {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        match self {
            Task::Plugin(task) => task.process(proc_info),
            Task::AudioSum(task) => task.process(proc_info),
            Task::NoteSum(task) => task.process(),
            Task::AutomationSum(task) => task.process(),
            Task::AudioDelayComp(task) => task.process(proc_info),
            Task::NoteDelayComp(task) => task.process(proc_info),
            Task::AutomationDelayComp(task) => task.process(proc_info),
            Task::UnloadedPlugin(task) => task.process(proc_info),
        }
    }
}
