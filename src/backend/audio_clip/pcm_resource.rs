static U24_TO_F32_RATIO: f32 = 2.0 / 0x00FFFFFF as f32;
static I16_TO_F32_RATIO: f32 = 1.0 / std::i16::MAX as f32;
static U8_TO_F32_RATIO: f32 = 2.0 / std::u8::MAX as f32;

pub enum PcmResource {
    F32(PcmF32),
    U24(PcmU24),
    I16(PcmI16),
    U8(PcmU8),
}

impl PcmResource {
    /// Fill the given buffer with this resource, starting from the frame `start_frame`.
    ///
    /// This will return the number of samples that were actually copied. This may be less than the
    /// length of `buf` if the end of the resource has been reached. Likewise, if `start_frame` is greater
    /// than or equal to the length of this resource, then `0` will be returned.
    pub fn fill_f32(&self, buf: &mut [f32], start_frame: usize) -> usize {
        match self {
            PcmResource::F32(resource) => {
                if start_frame >= resource.raw_data.len() {
                    // Out of range, return that no samples were copied.
                    return 0;
                }

                let len = buf.len().min(resource.raw_data.len() - start_frame);

                &mut buf[0..len]
                    .copy_from_slice(&resource.raw_data[start_frame..start_frame + len]);

                len
            }
            PcmResource::U24(resource) => {
                if start_frame >= resource.len {
                    // Out of range, return that no samples were copied.
                    return 0;
                }

                let len = buf.len().min(resource.len - start_frame);

                // Hint to the compiler that we want to optimize this loop. TODO: check if this
                // is even necessary.
                assert!((start_frame + len) * 3 <= resource.raw_data.len());

                // TODO: Check that the compiler optimizes this properly.
                for i in 0..len {
                    let f = (start_frame + i) * 3;

                    // TODO: Check that the endianness is right!!!!!!
                    let u32_val = (u32::from(resource.raw_data[f + 2])) << 16
                        | (u32::from(resource.raw_data[f + 1])) << 8
                        | (u32::from(resource.raw_data[f]));

                    buf[i] = (u32_val as f32 * U24_TO_F32_RATIO) - 1.0;
                }

                len
            }
            PcmResource::I16(resource) => {
                if start_frame >= resource.raw_data.len() {
                    // Out of range, return that no samples were copied.
                    return 0;
                }

                let len = buf.len().min(resource.raw_data.len() - start_frame);

                // TODO: Check that the compiler optimizes this properly.
                for i in 0..len {
                    buf[i] = f32::from(resource.raw_data[start_frame + i]) * I16_TO_F32_RATIO;
                }

                len
            }
            PcmResource::U8(resource) => {
                if start_frame >= resource.raw_data.len() {
                    // Out of range, return that no samples were copied.
                    return 0;
                }

                let len = buf.len().min(resource.raw_data.len() - start_frame);

                // TODO: Check that the compiler optimizes this properly.
                for i in 0..len {
                    buf[i] =
                        (f32::from(resource.raw_data[start_frame + i]) * U8_TO_F32_RATIO) - 1.0;
                }

                len
            }
        }
    }

    /// Returns the total number of frames in this resource.
    pub fn len(&self) -> usize {
        match self {
            PcmResource::F32(resource) => resource.raw_data.len(),
            PcmResource::U24(resource) => resource.len,
            PcmResource::I16(resource) => resource.raw_data.len(),
            PcmResource::U8(resource) => resource.raw_data.len(),
        }
    }

    /// Returns the sample rate of this resource.
    pub fn sample_rate(&self) -> f32 {
        match self {
            PcmResource::F32(resource) => resource.sample_rate,
            PcmResource::U24(resource) => resource.sample_rate,
            PcmResource::I16(resource) => resource.sample_rate,
            PcmResource::U8(resource) => resource.sample_rate,
        }
    }
}

pub struct PcmF32 {
    pub raw_data: Vec<f32>,
    pub sample_rate: f32,
}

pub struct PcmU24 {
    pub raw_data: Vec<u8>,
    pub sample_rate: f32,

    len: usize,
}

impl PcmU24 {
    /// Create a new `PcmU24` resource from raw bytes.
    ///
    /// Please note that the format of this buffer must be as follows:
    ///
    /// * Every sample consists of 3 bytes which represent an *unsigned* u32 integer
    /// from 0x00000000 to 0x00FFFFFF in little endian format. If your 24 bit data is signed or uses
    /// a different endianness, it must be converted first.
    /// * There is no fourth padding byte between samples. Each sample takes up exactly 3
    /// bytes.
    pub fn from_raw(raw_data: Vec<u8>, sample_rate: f32) -> Self {
        let len = raw_data.len() / 3;
        Self {
            raw_data,
            sample_rate,
            len,
        }
    }

    // TODO: Methods for creating a PcmU24 from different types of buffers.

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct PcmI16 {
    pub raw_data: Vec<i16>,
    pub sample_rate: f32,
}

pub struct PcmU8 {
    pub raw_data: Vec<u8>,
    pub sample_rate: f32,
}