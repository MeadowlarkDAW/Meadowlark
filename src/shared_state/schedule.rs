use basedrop::Shared;

use super::{resource_pool::ResourcePool, AudioGraphNode};

pub enum AudioGraphTask {
    Node {
        // We don't store the shared pointer to the node directly here because the pointer
        // to a node may change without the graph recompiling.
        node: Shared<Box<dyn AudioGraphNode>>,

        // We can store the shared pointers to the buffers because the graph always
        // recompiles when a pointer to the buffer changes.
        audio_through_buffers: Vec<Shared<Vec<f32>>>,
        extra_audio_in_buffers: Vec<Shared<Vec<f32>>>,
        extra_audio_out_buffers: Vec<Shared<Vec<f32>>>,
    },
    CopyAudioBuffer {
        src: Shared<Vec<f32>>,
        dst: Shared<Vec<f32>>,
    },
    // Mix the source audio buffer into the destination audio buffer.
    MixAudioBuffers {
        src: Shared<Vec<f32>>,
        dst: Shared<Vec<f32>>,
    }, // TODO: Delay compensation stuffs.
}

pub struct Schedule {
    pub master_out_buffers: [Shared<Vec<f32>>; 2],

    tasks: Vec<AudioGraphTask>,
    proc_info: ProcInfo,
}

impl Schedule {
    pub(super) fn new(
        tasks: Vec<AudioGraphTask>,
        sample_rate: f32,
        master_out_left: Shared<Vec<f32>>,
        master_out_right: Shared<Vec<f32>>,
    ) -> Self {
        Self {
            master_out_buffers: [master_out_left, master_out_right],
            tasks,
            proc_info: ProcInfo {
                frames: 0,
                sample_rate,
                sample_rate_recip: 1.0 / sample_rate,
            },
        }
    }

    /// Only to be used by the rt thread.
    pub(super) fn process(&mut self, frames: usize) {
        // TODO: Use multithreading for processing tasks.

        self.proc_info.frames = frames;

        // Where the magic happens!
        for task in self.tasks.iter_mut() {
            match task {
                AudioGraphTask::Node {
                    node,
                    audio_through_buffers,
                    extra_audio_in_buffers,
                    extra_audio_out_buffers,
                } => {
                    // This should not panic because the rt thread is the only place these nodes
                    // are mutated.
                    let node = Shared::get_mut(node).unwrap();

                    node.process(
                        &self.proc_info,
                        audio_through_buffers,
                        extra_audio_in_buffers,
                        extra_audio_out_buffers,
                    );
                }
                AudioGraphTask::CopyAudioBuffer { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are mutated.
                    //
                    // TODO: Find a way to do this more ergonomically and efficiently, perhaps by
                    // using a safe wrapper around a custom type?
                    Shared::get_mut(dst).unwrap().copy_from_slice(src);
                }
                AudioGraphTask::MixAudioBuffers { src, dst } => {
                    // This should not panic because the rt thread is the only place these buffers
                    // are mutated.
                    //
                    // TODO: Find a way to do this more ergonomically and efficiently, perhaps by
                    // using a safe wrapper around a custom type?
                    let dst = Shared::get_mut(dst).unwrap();

                    for i in 0..frames {
                        // Safe because the scheduler calling this method ensures that all buffers
                        // have the length `proc_info.frames`.
                        //
                        // TODO: Find a more ergonomic way to do this using a safe wrapper around a
                        // custom type? We also want to make it so a buffer can never be resized except
                        // by this scheduler at the top of this loop.
                        unsafe {
                            *dst.get_unchecked_mut(i) += *src.get_unchecked(i);
                        }
                    }
                }
            }
        }
    }
}

pub struct ProcInfo {
    /// The number of frames in every audio buffer.
    pub frames: usize,

    /// The sample rate of the stream. This remains constant for the whole lifetime of this node,
    /// so this is just provided for convenience.
    pub sample_rate: f32,

    /// The recipricol of the sample rate (1.0 / sample_rate) of the stream. This remains constant
    /// for the whole lifetime of this node, so this is just provided for convenience.
    pub sample_rate_recip: f32,
}
