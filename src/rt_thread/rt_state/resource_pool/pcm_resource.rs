pub trait PcmResource: Send {
    /// The number of channels in this resource.
    fn channels(&self) -> usize;
    /// The length of this resource in frames.
    fn frames(&self) -> usize;
    /// The sample rate of this resource.
    fn sample_rate(&self) -> u32;

    /// Copy samples from a specific channel into the given buffer, starting from the given frame.
    ///
    /// This will return the number of samples that were actually copied to the buffer. This may be less
    /// than the length of the buffer (or even 0) if the end of the resource's data is reached.
    ///
    /// This will return an error if the channel does not exist in this resource. There is garaunteed to always
    /// be at-least one channel.
    fn copy_channel_f32(
        &self,
        frame: usize,
        channel: usize,
        buffer: &mut [f32],
    ) -> Result<usize, ()>;

    /// Copy samples from the first channel into the given buffer, starting from the given frame.
    ///
    /// This will return the number of samples that were actually copied to the buffer. This may be less
    /// than the length of the buffer (or even 0) if the end of the resource's data is reached.
    ///
    /// It is garaunteed that at-least one channel exists.
    fn copy_mono_f32(&self, frame: usize, channel: usize, buffer: &mut [f32]) -> usize;

    /// Copy samples from a specific channel into the given left and right buffers, starting from the given frame.
    /// This can be more efficient than copying both channels individually.
    ///
    /// If the resource is mono, then the mono channel will be copied to both buffers. If the number of channels is
    /// two or more, then only the first two channels will be copied.
    ///
    /// This will return the number of samples that were actually copied to the buffer. This may be less
    /// than the length of the buffer (or even 0) if the end of the resource's data is reached.
    ///
    /// This will return an error if `buffer_left` and `buffer_right` are not the same length.
    fn copy_stereo_f32(
        &self,
        frame: usize,
        buffer_left: &mut [f32],
        buffer_right: &mut [f32],
    ) -> Result<usize, ()>;

    // TODO: Copy into 64 bit floating point buffers
}

pub struct PcmResourceF32 {
    data: Vec<Vec<f32>>,
    sample_rate: u32,
}

impl PcmResourceF32 {
    /// Creates a new PCM resource of f32 samples.
    ///
    /// This will return an error if data is empty or if all channels are not the
    /// same length.
    pub fn new(data: Vec<Vec<f32>>, sample_rate: u32) -> Result<Self, ()> {
        if data.is_empty() {
            return Err(());
        } else {
            let len = data[0].len();
            if len == 0 {
                return Err(());
            }
            for ch in data.iter().skip(1) {
                if ch.len() != len {
                    return Err(());
                }
            }
        }

        Ok(Self { data, sample_rate })
    }
}

impl PcmResource for PcmResourceF32 {
    fn channels(&self) -> usize {
        self.data.len()
    }
    fn frames(&self) -> usize {
        self.data[0].len()
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn copy_channel_f32(
        &self,
        frame: usize,
        channel: usize,
        buffer: &mut [f32],
    ) -> Result<usize, ()> {
        if let Some(ch) = self.data.get(channel) {
            if frame < ch.len() {
                let len = buffer.len().min(ch.len() - frame);
                &mut buffer[0..len].copy_from_slice(&ch[0..len]);
                Ok(len)
            } else {
                // frame lies outside range of data
                Ok(0)
            }
        } else {
            Err(())
        }
    }

    fn copy_mono_f32(&self, frame: usize, channel: usize, buffer: &mut [f32]) -> usize {
        // There is garaunteed to be at-least one channel
        self.copy_channel_f32(frame, channel, buffer).unwrap()
    }

    fn copy_stereo_f32(
        &self,
        frame: usize,
        buffer_left: &mut [f32],
        buffer_right: &mut [f32],
    ) -> Result<usize, ()> {
        if buffer_left.len() != buffer_right.len() {
            return Err(());
        }

        if frame >= self.data[0].len() {
            // frame lies outside range of data
            return Ok(0);
        }

        let len = buffer_left.len().min(self.data[0].len() - frame);

        if self.data.len() == 1 {
            // Copy mono channel into both buffers.
            &mut buffer_left[0..len].copy_from_slice(&self.data[0][0..len]);
            &mut buffer_right[0..len].copy_from_slice(&self.data[0][0..len]);
        } else {
            &mut buffer_left[0..len].copy_from_slice(&self.data[0][0..len]);
            &mut buffer_right[0..len].copy_from_slice(&self.data[1][0..len]);
        }

        Ok(len)
    }
}

// TODO: 64 bit floating-point samples
/*
pub struct PcmResourceF64 {
    data: Vec<Vec<f64>>,
}
*/

// TODO: 24 bit samples
/*
pub struct PcmResourceI24 {
    data: Vec<Vec<u8>>,
}
*/

pub struct PcmResourceI16 {
    data: Vec<Vec<i16>>,
    sample_rate: u32,
}

impl PcmResourceI16 {
    /// Creates a new PCM resource of i16 samples.
    ///
    /// This will return an error if data is empty or if all channels are not the
    /// same length.
    pub fn new(data: Vec<Vec<i16>>, sample_rate: u32) -> Result<Self, ()> {
        if data.is_empty() {
            return Err(());
        } else {
            let len = data[0].len();
            if len == 0 {
                return Err(());
            }
            for ch in data.iter().skip(1) {
                if ch.len() != len {
                    return Err(());
                }
            }
        }

        Ok(Self { data, sample_rate })
    }
}

impl PcmResource for PcmResourceI16 {
    fn channels(&self) -> usize {
        self.data.len()
    }
    fn frames(&self) -> usize {
        self.data[0].len()
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn copy_channel_f32(
        &self,
        frame: usize,
        channel: usize,
        buffer: &mut [f32],
    ) -> Result<usize, ()> {
        if let Some(ch) = self.data.get(channel) {
            if frame < ch.len() {
                let len = buffer.len().min(ch.len() - frame);

                // TODO: Check if the compiler elids all bounds checking in these loops. If it doesn't,
                // we can safely use unsafe indexing here.

                // TODO: Manual SIMD optimizations?

                for i in 0..len {
                    buffer[i] = f32::from(ch[frame + i]);
                }
                Ok(len)
            } else {
                // frame lies outside range of data
                Ok(0)
            }
        } else {
            Err(())
        }
    }

    fn copy_mono_f32(&self, frame: usize, channel: usize, buffer: &mut [f32]) -> usize {
        // There is garaunteed to be at-least one channel
        self.copy_channel_f32(frame, channel, buffer).unwrap()
    }

    fn copy_stereo_f32(
        &self,
        frame: usize,
        buffer_left: &mut [f32],
        buffer_right: &mut [f32],
    ) -> Result<usize, ()> {
        if buffer_left.len() != buffer_right.len() {
            return Err(());
        }

        if frame >= self.data[0].len() {
            // frame lies outside range of data
            return Ok(0);
        }

        let len = buffer_left.len().min(self.data[0].len() - frame);

        // TODO: Check if the compiler elids all bounds checking in these loops. If it doesn't,
        // we can safely use unsafe indexing here.

        // TODO: Manual SIMD optimizations?

        if self.data.len() == 1 {
            // Copy mono channel into both buffers.

            for i in 0..len {
                buffer_left[i] = f32::from(self.data[0][frame + i]);
                buffer_right[i] = f32::from(self.data[0][frame + i]);
            }
        } else {
            for i in 0..len {
                buffer_left[i] = f32::from(self.data[0][frame + i]);
                buffer_right[i] = f32::from(self.data[1][frame + i]);
            }
        }

        Ok(len)
    }
}

// TODO: 8 bit samples
/*
pub struct PcmResourceU8 {
    data: Vec<Vec<u8>>,
}
*/
