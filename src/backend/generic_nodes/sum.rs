use crate::backend::graph::{clear_audio_outputs, AudioBlockBuffer, AudioGraphNode, ProcInfo};
use crate::backend::timeline::TimelineTransport;

pub struct MonoSumNode {
    num_inputs: usize,
}

impl MonoSumNode {
    pub fn new(num_inputs: usize) -> Self {
        Self { num_inputs }
    }
}

impl AudioGraphNode for MonoSumNode {
    fn audio_in_ports(&self) -> usize {
        self.num_inputs
    }
    fn audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        if audio_out.is_empty() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let frames = proc_info.frames();

        let dst = &mut audio_out[0];

        // TODO: SIMD

        match audio_in.len() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            0 => dst.clear_frames(frames),
            1 => {
                // Just copy.
                dst.copy_frames_from(&audio_in[0], frames);
            }
            2 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i] + audio_in[1][i];
                }
            }
            3 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i] + audio_in[1][i] + audio_in[2][i];
                }
            }
            4 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i] + audio_in[1][i] + audio_in[2][i] + audio_in[3][i];
                }
            }
            5 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i]
                        + audio_in[1][i]
                        + audio_in[2][i]
                        + audio_in[3][i]
                        + audio_in[4][i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i]
                        + audio_in[1][i]
                        + audio_in[2][i]
                        + audio_in[3][i]
                        + audio_in[4][i]
                        + audio_in[5][i];
                }
            }
            7 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i]
                        + audio_in[1][i]
                        + audio_in[2][i]
                        + audio_in[3][i]
                        + audio_in[4][i]
                        + audio_in[5][i]
                        + audio_in[6][i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst[i] = audio_in[0][i]
                        + audio_in[1][i]
                        + audio_in[2][i]
                        + audio_in[3][i]
                        + audio_in[4][i]
                        + audio_in[5][i]
                        + audio_in[6][i]
                        + audio_in[7][i];
                }
            }
            num_inputs => {
                // Copy the first buffer.
                dst.copy_frames_from(&audio_in[0], frames);

                for ch_i in 1..num_inputs {
                    let src = &audio_in[ch_i];

                    for i in 0..frames {
                        dst[i] += src[i];
                    }
                }
            }
        }
    }
}

pub struct StereoSumNode {
    num_stereo_inputs: usize,
}

impl StereoSumNode {
    pub fn new(num_stereo_inputs: usize) -> Self {
        Self { num_stereo_inputs }
    }
}

impl AudioGraphNode for StereoSumNode {
    fn audio_in_ports(&self) -> usize {
        self.num_stereo_inputs * 2
    }
    fn audio_out_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        // Assume the host always connects ports in a stereo pair together.
        if audio_out.len() < 2 {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let frames = proc_info.frames();

        let dst_left = &mut audio_out[0];
        let dst_right = &mut audio_out[1];

        // TODO: SIMD

        match audio_in.len() {
            0 | 1 => {
                // As per the spec, all unused audio output buffers must be cleared to 0.0.
                clear_audio_outputs(audio_out, proc_info)
            }
            2 => {
                // Just copy.
                dst_left.copy_frames_from(&audio_in[0], frames);
                dst_right.copy_frames_from(&audio_in[1], frames);
            }
            4 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i] + audio_in[2][i];
                    dst_right[i] = audio_in[1][i] + audio_in[3][i];
                }
            }
            6 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i] + audio_in[2][i] + audio_in[4][i];
                    dst_right[i] = audio_in[1][i] + audio_in[3][i] + audio_in[5][i];
                }
            }
            8 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i] + audio_in[2][i] + audio_in[4][i] + audio_in[6][i];
                    dst_right[i] =
                        audio_in[1][i] + audio_in[3][i] + audio_in[5][i] + audio_in[7][i];
                }
            }
            10 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i]
                        + audio_in[2][i]
                        + audio_in[4][i]
                        + audio_in[6][i]
                        + audio_in[8][i];
                    dst_right[i] = audio_in[1][i]
                        + audio_in[3][i]
                        + audio_in[5][i]
                        + audio_in[7][i]
                        + audio_in[9][i];
                }
            }
            12 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i]
                        + audio_in[2][i]
                        + audio_in[4][i]
                        + audio_in[6][i]
                        + audio_in[8][i]
                        + audio_in[10][i];
                    dst_right[i] = audio_in[1][i]
                        + audio_in[3][i]
                        + audio_in[5][i]
                        + audio_in[7][i]
                        + audio_in[9][i]
                        + audio_in[11][i];
                }
            }
            14 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i]
                        + audio_in[2][i]
                        + audio_in[4][i]
                        + audio_in[6][i]
                        + audio_in[8][i]
                        + audio_in[10][i]
                        + audio_in[12][i];
                    dst_right[i] = audio_in[1][i]
                        + audio_in[3][i]
                        + audio_in[5][i]
                        + audio_in[7][i]
                        + audio_in[9][i]
                        + audio_in[11][i]
                        + audio_in[13][i];
                }
            }
            16 => {
                for i in 0..frames {
                    dst_left[i] = audio_in[0][i]
                        + audio_in[2][i]
                        + audio_in[4][i]
                        + audio_in[6][i]
                        + audio_in[8][i]
                        + audio_in[10][i]
                        + audio_in[12][i]
                        + audio_in[14][i];
                    dst_right[i] = audio_in[1][i]
                        + audio_in[3][i]
                        + audio_in[5][i]
                        + audio_in[7][i]
                        + audio_in[9][i]
                        + audio_in[11][i]
                        + audio_in[13][i]
                        + audio_in[15][i];
                }
            }
            num_inputs => {
                let num_stereo_inputs = num_inputs / 2;

                // Assume the host always connects ports in a stereo pair together. But to be
                // safe, make sure that the number of inputs is a power of 2.
                if num_stereo_inputs == 0 || num_inputs & 1 == 1 {
                    // Second half is equivalent to `num_inputs % 2`.
                    // As per the spec, all unused audio output buffers must be cleared to 0.0.
                    clear_audio_outputs(audio_out, proc_info);
                    return;
                }

                // Copy the first channel.
                dst_left.copy_frames_from(&audio_in[0], frames);
                dst_right.copy_frames_from(&audio_in[1], frames);

                // Add the rest of the channels.
                for ch_i in 1..num_stereo_inputs {
                    // Safe because we checked that the number of inputs is a power of 2.
                    let src_left = unsafe { audio_in.get_unchecked(ch_i * 2) };
                    let src_right = unsafe { audio_in.get_unchecked((ch_i * 2) + 1) };

                    for i in 0..frames {
                        dst_left[i] += src_left[i];
                        dst_right[i] += src_right[i];
                    }
                }
            }
        }
    }
}
