use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use smallvec::SmallVec;
use std::fmt::{Debug, Formatter};

pub use clack_host::events::io::{EventBuffer, EventBufferIter};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugBufferType {
    Audio32,
    Audio64,
    Event,
    Note,
}

impl Debug for DebugBufferType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugBufferType::Audio32 => f.write_str("fl"),
            DebugBufferType::Audio64 => f.write_str("db"),
            DebugBufferType::Event => f.write_str("ev"),
            DebugBufferType::Note => f.write_str("nt"),
        }
    }
}

/// Used for debugging and verifying purposes.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DebugBufferID {
    pub index: u32,
    pub buffer_type: DebugBufferType,
}

impl Debug for DebugBufferID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.buffer_type, self.index)
    }
}

pub struct BufferInner<T: Clone + Copy + Send + Sync + 'static> {
    pub data: Vec<T>,
    pub is_constant: bool,
}

impl<T: Clone + Copy + Send + Sync + 'static> BufferInner<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity), is_constant: false }
    }
}

impl<T: Clone + Copy + Send + Sync + Default + 'static> BufferInner<T> {
    fn new(max_frames: usize) -> Self {
        Self { data: vec![T::default(); max_frames], is_constant: false }
    }
}

struct Buffer<T: Clone + Copy + Send + Sync + 'static> {
    data: AtomicRefCell<BufferInner<T>>,
    debug_info: DebugBufferID,
}

impl<T: Clone + Copy + Send + Sync + 'static> Buffer<T> {}

pub struct SharedBuffer<T: Clone + Copy + Send + Sync + 'static> {
    buffer: Shared<Buffer<T>>,
}

impl<T: Clone + Copy + Send + Sync + 'static> SharedBuffer<T> {
    pub fn with_capacity(
        capacity: usize,
        debug_info: DebugBufferID,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        Self {
            buffer: Shared::new(
                coll_handle,
                Buffer {
                    data: AtomicRefCell::new(BufferInner::with_capacity(capacity)),
                    debug_info,
                },
            ),
        }
    }

    #[inline]
    pub fn borrow(&self) -> AtomicRef<BufferInner<T>> {
        self.buffer.data.borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> AtomicRefMut<BufferInner<T>> {
        self.buffer.data.borrow_mut()
    }

    #[inline]
    pub fn id(&self) -> DebugBufferID {
        self.buffer.debug_info
    }

    pub fn truncate(&self) {
        self.borrow_mut().data.truncate(0)
    }
}

impl<T: Clone + Copy + Send + Sync + 'static + Default> SharedBuffer<T> {
    pub fn new(
        max_frames: usize,
        debug_info: DebugBufferID,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        Self {
            buffer: Shared::new(
                coll_handle,
                Buffer { data: AtomicRefCell::new(BufferInner::new(max_frames)), debug_info },
            ),
        }
    }

    pub fn clear(&self, frames: usize) {
        let mut buf_ref = self.borrow_mut();
        let frames = frames.min(buf_ref.data.len());

        buf_ref.data[0..frames].fill(T::default());
    }

    pub fn clear_and_set_constant_hint(&self, frames: usize) {
        let mut buf_ref = self.borrow_mut();
        let frames = frames.min(buf_ref.data.len());

        buf_ref.data[0..frames].fill(T::default());
        buf_ref.is_constant = true;
    }
}

impl<T: Copy + Default + PartialEq + Send + Sync + 'static> SharedBuffer<T> {
    /// Checks if the buffer could be possibly silent, without reading the whole buffer.
    ///
    /// This only relies on the `is_constant` flag and the first sample of the buffer, and thus
    /// may not be accurate.
    pub fn has_silent_hint(&self) -> bool {
        let b = self.borrow();
        b.is_constant && b.data[0] == T::default()
    }
}

impl<T: Clone + Copy + Send + Sync + 'static> Clone for SharedBuffer<T> {
    fn clone(&self) -> Self {
        Self { buffer: Shared::clone(&self.buffer) }
    }
}

#[allow(unused)]
pub enum RawAudioChannelBuffers {
    F32(SmallVec<[SharedBuffer<f32>; 2]>),
    F64(SmallVec<[SharedBuffer<f64>; 2]>),
}

impl Debug for RawAudioChannelBuffers {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self {
            RawAudioChannelBuffers::F32(buffers) => {
                f.debug_list().entries(buffers.iter().map(|b| b.id())).finish()
            }
            RawAudioChannelBuffers::F64(buffers) => {
                f.debug_list().entries(buffers.iter().map(|b| b.id())).finish()
            }
        }
    }
}

pub enum AudioBufferType<'a> {
    F32(AtomicRef<'a, Vec<f32>>),
    F64(AtomicRef<'a, Vec<f64>>),
}

pub enum AudioBufferTypeMut<'a> {
    F32(AtomicRefMut<'a, Vec<f32>>),
    F64(AtomicRefMut<'a, Vec<f64>>),
}

pub type BufferRef<'a, T> = AtomicRef<'a, BufferInner<T>>;
pub type BufferRefMut<'a, T> = AtomicRefMut<'a, BufferInner<T>>;

pub struct AudioPortBuffer {
    pub _raw_channels: RawAudioChannelBuffers,
    channels: usize,
    latency: u32,
}

impl Debug for AudioPortBuffer {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self._raw_channels.fmt(f)
    }
}

impl AudioPortBuffer {
    pub fn _new(buffers: SmallVec<[SharedBuffer<f32>; 2]>, latency: u32) -> Self {
        let channels = buffers.len();

        Self { _raw_channels: RawAudioChannelBuffers::F32(buffers), latency, channels }
    }

    pub fn latency(&self) -> u32 {
        self.latency
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Checks if all channel buffers could be possibly silent, without reading the whole buffers.
    ///
    /// This only relies on the `is_constant` flag and the first sample of each buffer, and thus
    /// may not be accurate.
    pub fn has_silent_hint(&self) -> bool {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(channels) => channels.iter().all(|c| c.has_silent_hint()),
            RawAudioChannelBuffers::F64(channels) => channels.iter().all(|c| c.has_silent_hint()),
        }
    }

    pub fn is_silent(&self, frames: usize) -> bool {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(buffers) => {
                for buf in buffers.iter() {
                    let buf = buf.borrow();
                    let buf = &buf.data[0..frames.min(buf.data.len())];
                    for x in buf.iter() {
                        if *x != 0.0 {
                            return false;
                        }
                    }
                }
            }
            RawAudioChannelBuffers::F64(buffers) => {
                for buf in buffers.iter() {
                    let buf = buf.borrow();
                    let buf = &buf.data[0..frames.min(buf.data.len())];
                    for x in buf.iter() {
                        if *x != 0.0 {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn channel_f32(&self, index: usize) -> Option<BufferRef<f32>> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => b.get(index).map(|b| b.borrow()),
            _ => None,
        }
    }

    pub fn mono_f32(&self) -> Option<BufferRef<f32>> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => Some(b[0].borrow()),
            _ => None,
        }
    }

    pub fn stereo_f32(&self) -> Option<(BufferRef<f32>, BufferRef<f32>)> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => {
                if b.len() > 1 {
                    Some((b[0].borrow(), b[1].borrow()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn iter_f32(&self) -> Option<impl Iterator<Item = AtomicRef<'_, BufferInner<f32>>>> {
        if let RawAudioChannelBuffers::F32(b) = &self._raw_channels {
            Some(b.iter().map(|b| b.borrow()))
        } else {
            None
        }
    }

    // TODO: Helper methods to retrieve more than 2 channels at once
}

pub struct AudioPortBufferMut {
    pub _raw_channels: RawAudioChannelBuffers,
    channels: usize,
    latency: u32,
}

impl Debug for AudioPortBufferMut {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self._raw_channels.fmt(f)
    }
}

impl AudioPortBufferMut {
    pub fn _new(buffers: SmallVec<[SharedBuffer<f32>; 2]>, latency: u32) -> Self {
        let channels = buffers.len();

        Self { _raw_channels: RawAudioChannelBuffers::F32(buffers), latency, channels }
    }

    pub fn latency(&self) -> u32 {
        self.latency
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn channel_f32(&self, index: usize) -> Option<BufferRef<f32>> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => b.get(index).map(|b| b.borrow()),
            _ => None,
        }
    }

    pub fn channel_f32_mut(&mut self, index: usize) -> Option<BufferRefMut<f32>> {
        match &mut self._raw_channels {
            RawAudioChannelBuffers::F32(b) => b.get(index).map(|b| b.borrow_mut()),
            _ => None,
        }
    }

    pub fn mono_f32(&self) -> Option<BufferRef<f32>> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => Some(b[0].borrow()),
            _ => None,
        }
    }

    pub fn mono_f32_mut(&mut self) -> Option<BufferRefMut<f32>> {
        match &mut self._raw_channels {
            RawAudioChannelBuffers::F32(b) => Some(b[0].borrow_mut()),
            _ => None,
        }
    }

    pub fn stereo_f32(&self) -> Option<(BufferRef<f32>, BufferRef<f32>)> {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(b) => {
                if b.len() > 1 {
                    Some((b[0].borrow(), b[1].borrow()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn stereo_f32_mut(&mut self) -> Option<(BufferRefMut<f32>, BufferRefMut<f32>)> {
        match &mut self._raw_channels {
            RawAudioChannelBuffers::F32(b) => {
                if b.len() > 1 {
                    Some((b[0].borrow_mut(), b[1].borrow_mut()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Checks if all channel buffers could be possibly silent, without reading the whole buffers.
    ///
    /// This only relies on the `is_constant` flag and the first sample of each buffer, and thus
    /// may not be accurate.
    pub fn has_silent_hint(&self) -> bool {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(channels) => channels.iter().all(|c| c.has_silent_hint()),
            RawAudioChannelBuffers::F64(channels) => channels.iter().all(|c| c.has_silent_hint()),
        }
    }

    pub fn is_silent(&self, frames: usize) -> bool {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(buffers) => {
                for buf in buffers {
                    let buf = buf.borrow();
                    let buf = &buf.data[0..frames.min(buf.data.len())];
                    for x in buf.iter() {
                        if *x != 0.0 {
                            return false;
                        }
                    }
                }
            }
            RawAudioChannelBuffers::F64(buffers) => {
                for buf in buffers {
                    let buf = buf.borrow();
                    let buf = &buf.data[0..frames.min(buf.data.len())];
                    for x in buf.iter() {
                        if *x != 0.0 {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    /// Clear the number of frames in all channels.
    ///
    /// Note this does not set the constant hint.
    pub fn clear_all(&mut self, frames: usize) {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(buffers) => {
                for buf in buffers {
                    buf.clear(frames);
                }
            }
            RawAudioChannelBuffers::F64(buffers) => {
                for buf in buffers {
                    buf.clear(frames);
                }
            }
        }
    }

    /// Clear the number of frames in all channels, and also
    /// set the constant hint to `true` for all channels.
    pub fn clear_all_and_set_constant_hint(&mut self, frames: usize) {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(buffers) => {
                for buf in buffers {
                    buf.clear_and_set_constant_hint(frames);
                }
            }
            RawAudioChannelBuffers::F64(buffers) => {
                for buf in buffers {
                    buf.clear_and_set_constant_hint(frames);
                }
            }
        }
    }

    pub fn set_constant_hint(&mut self, is_constant: bool) {
        match &self._raw_channels {
            RawAudioChannelBuffers::F32(buffers) => {
                for buf in buffers {
                    buf.borrow_mut().is_constant = is_constant;
                }
            }
            RawAudioChannelBuffers::F64(buffers) => {
                for buf in buffers {
                    buf.borrow_mut().is_constant = is_constant;
                }
            }
        }
    }

    pub fn iter_f32(&self) -> Option<impl Iterator<Item = AtomicRef<'_, BufferInner<f32>>>> {
        if let RawAudioChannelBuffers::F32(b) = &self._raw_channels {
            Some(b.iter().map(|b| b.borrow()))
        } else {
            None
        }
    }

    pub fn iter_f32_mut(
        &mut self,
    ) -> Option<impl Iterator<Item = AtomicRefMut<'_, BufferInner<f32>>>> {
        if let RawAudioChannelBuffers::F32(b) = &mut self._raw_channels {
            Some(b.iter_mut().map(|b| b.borrow_mut()))
        } else {
            None
        }
    }

    // TODO: Helper methods to retrieve more than 2 channels at once
}
