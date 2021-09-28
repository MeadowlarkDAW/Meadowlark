use crate::backend::graph::{AudioGraphNode, ProcBuffers, ProcInfo};
use crate::backend::timeline::TimelineTransport;

pub struct MonoSumNode {
    num_inputs: u32,
}

impl MonoSumNode {
    pub fn new(num_inputs: u32) -> Self {
        Self { num_inputs }
    }
}

impl AudioGraphNode for MonoSumNode {
    fn mono_audio_in_ports(&self) -> u32 {
        self.num_inputs
    }
    fn mono_audio_out_ports(&self) -> u32 {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if buffers.mono_audio_out.is_empty() {
            return;
        }

        let frames = proc_info.frames();

        // Won't panic because we checked this was not empty earlier.
        let dst = &mut *buffers.mono_audio_out.buffer_mut(0).unwrap();

        let audio_in = &buffers.mono_audio_in;

        // TODO: SIMD

        match audio_in.len() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            0 => dst.clear_frames(frames),
            1 => {
                // Just copy.
                dst.copy_frames_from(&*audio_in.buffer(0).unwrap(), frames);
            }
            2 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i] + src_2[i];
                }
            }
            3 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i] + src_2[i] + src_3[i];
                }
            }
            4 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i] + src_2[i] + src_3[i] + src_4[i];
                }
            }
            5 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i] + src_2[i] + src_3[i] + src_4[i] + src_5[i];
                }
            }
            6 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i] + src_2[i] + src_3[i] + src_4[i] + src_5[i] + src_6[i];
                }
            }
            7 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();
                let src_7 = audio_in.buffer(6).unwrap();

                for i in 0..frames {
                    dst[i] =
                        src_1[i] + src_2[i] + src_3[i] + src_4[i] + src_5[i] + src_6[i] + src_7[i];
                }
            }
            8 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();
                let src_7 = audio_in.buffer(6).unwrap();
                let src_8 = audio_in.buffer(7).unwrap();

                for i in 0..frames {
                    dst[i] = src_1[i]
                        + src_2[i]
                        + src_3[i]
                        + src_4[i]
                        + src_5[i]
                        + src_6[i]
                        + src_7[i]
                        + src_8[i];
                }
            }
            // TODO: Additional optimized loops?
            num_inputs => {
                // Copy the first buffer.
                dst.copy_frames_from(&*audio_in.buffer(0).unwrap(), frames);

                for ch_i in 1..num_inputs {
                    let src = audio_in.buffer(ch_i).unwrap();

                    for i in 0..frames {
                        dst[i] += src[i];
                    }
                }
            }
        }
    }
}

pub struct StereoSumNode {
    num_stereo_inputs: u32,
}

impl StereoSumNode {
    pub fn new(num_stereo_inputs: u32) -> Self {
        Self { num_stereo_inputs }
    }
}

impl AudioGraphNode for StereoSumNode {
    fn stereo_audio_in_ports(&self) -> u32 {
        self.num_stereo_inputs
    }
    fn stereo_audio_out_ports(&self) -> u32 {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if buffers.stereo_audio_out.is_empty() {
            return;
        }

        let frames = proc_info.frames();

        // Won't panic because we checked this was not empty earlier.
        let dst = &mut *buffers.stereo_audio_out.buffer_mut(0).unwrap();

        let audio_in = &buffers.stereo_audio_in;

        // TODO: SIMD

        match audio_in.len() {
            0 => {
                // As per the spec, all unused audio output buffers must be cleared to 0.0.
                dst.clear_frames(frames);
            }
            1 => {
                // Just copy.
                dst.copy_frames_from(&*audio_in.buffer(0).unwrap(), frames);
            }
            2 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i] + src_2.left[i];
                    dst.right[i] = src_1.right[i] + src_2.right[i];
                }
            }
            3 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i] + src_2.left[i] + src_3.left[i];
                    dst.right[i] = src_1.right[i] + src_2.right[i] + src_3.right[i];
                }
            }
            4 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i] + src_2.left[i] + src_3.left[i] + src_4.left[i];
                    dst.right[i] =
                        src_1.right[i] + src_2.right[i] + src_3.right[i] + src_4.right[i];
                }
            }
            5 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i]
                        + src_2.left[i]
                        + src_3.left[i]
                        + src_4.left[i]
                        + src_5.left[i];
                    dst.right[i] = src_1.right[i]
                        + src_2.right[i]
                        + src_3.right[i]
                        + src_4.right[i]
                        + src_5.right[i];
                }
            }
            6 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i]
                        + src_2.left[i]
                        + src_3.left[i]
                        + src_4.left[i]
                        + src_5.left[i]
                        + src_6.left[i];
                    dst.right[i] = src_1.right[i]
                        + src_2.right[i]
                        + src_3.right[i]
                        + src_4.right[i]
                        + src_5.right[i]
                        + src_6.right[i];
                }
            }
            7 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();
                let src_7 = audio_in.buffer(6).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i]
                        + src_2.left[i]
                        + src_3.left[i]
                        + src_4.left[i]
                        + src_5.left[i]
                        + src_6.left[i]
                        + src_7.left[i];
                    dst.right[i] = src_1.right[i]
                        + src_2.right[i]
                        + src_3.right[i]
                        + src_4.right[i]
                        + src_5.right[i]
                        + src_6.right[i]
                        + src_7.right[i];
                }
            }
            8 => {
                let src_1 = audio_in.buffer(0).unwrap();
                let src_2 = audio_in.buffer(1).unwrap();
                let src_3 = audio_in.buffer(2).unwrap();
                let src_4 = audio_in.buffer(3).unwrap();
                let src_5 = audio_in.buffer(4).unwrap();
                let src_6 = audio_in.buffer(5).unwrap();
                let src_7 = audio_in.buffer(6).unwrap();
                let src_8 = audio_in.buffer(7).unwrap();

                for i in 0..frames {
                    dst.left[i] = src_1.left[i]
                        + src_2.left[i]
                        + src_3.left[i]
                        + src_4.left[i]
                        + src_5.left[i]
                        + src_6.left[i]
                        + src_7.left[i]
                        + src_8.left[i];
                    dst.right[i] = src_1.right[i]
                        + src_2.right[i]
                        + src_3.right[i]
                        + src_4.right[i]
                        + src_5.right[i]
                        + src_6.right[i]
                        + src_7.right[i]
                        + src_8.right[i];
                }
            }
            // TODO: Additional optimized loops?
            num_stereo_inputs => {
                // Copy the first channel.
                dst.copy_frames_from(&*audio_in.buffer(0).unwrap(), frames);

                // Add the rest of the channels.
                for ch_i in 1..num_stereo_inputs {
                    let src = audio_in.buffer(ch_i).unwrap();

                    for i in 0..frames {
                        dst.left[i] += src.left[i];
                        dst.right[i] += src.right[i];
                    }
                }
            }
        }
    }
}
