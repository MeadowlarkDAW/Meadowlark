use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use num_traits::Num;

use super::{AudioGraphNode, MonoBlockBuffer, ProcInfo, StereoBlockBuffer};

pub struct AudioGraphTask {
    pub node: Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>,
    pub proc_buffers: ProcBuffers<f32>,
}

/// An abstraction that prepares buffers into a nice format for nodes.
pub struct ProcBuffers<T: Num + Copy + Clone> {
    /// Mono audio input buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected.
    pub mono_audio_in: MonoProcBuffers<T>,

    /// Mono audio output buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    pub mono_audio_out: MonoProcBuffersMut<T>,

    /// Stereo audio input buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected.
    pub stereo_audio_in: StereoProcBuffers<T>,

    /// Stereo audio output buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    pub stereo_audio_out: StereoProcBuffersMut<T>,
}

impl<T: Num + Copy + Clone> ProcBuffers<T> {
    /// Clears all output buffers to 0.0.
    ///
    /// This exists because audio output buffers may not be cleared to 0.0 before being sent
    /// to a node. As such, this spec requires all unused audio output buffers to be manually
    /// cleared by the node itself. This is provided as a convenience method.
    pub fn clear_audio_out_buffers(&mut self, proc_info: &ProcInfo) {
        let frames = proc_info.frames();

        for b in self.mono_audio_out.buffers.iter() {
            // This should not panic because the rt thread is the only place these buffers
            // are borrowed.
            //
            // TODO: Use unsafe instead of runtime checking? It would be more efficient,
            // but in theory a bug in the scheduler could try and assign the same buffer
            // twice in the same task or in parallel tasks, so it would be nice to
            // detect if that happens.
            (&mut *AtomicRefCell::borrow_mut(&b.0)).clear_frames(frames);
        }

        for b in self.stereo_audio_out.buffers.iter() {
            // This should not panic because the rt thread is the only place these buffers
            // are borrowed.
            //
            // TODO: Use unsafe instead of runtime checking? It would be more efficient,
            // but in theory a bug in the scheduler could try and assign the same buffer
            // twice in the same task or in parallel tasks, so it would be nice to
            // detect if that happens.
            (&mut *AtomicRefCell::borrow_mut(&b.0)).clear_frames(frames);
        }
    }
}

/// Mono audio input buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do ***NOT*** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct MonoProcBuffers<T: Num + Copy + Clone> {
    buffers: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<T>>>, usize)>,
}

impl<T: Num + Copy + Clone> MonoProcBuffers<T> {
    pub(crate) fn new(buffers: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<T>>>, usize)>) -> Self {
        Self { buffers }
    }

    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers in this list is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// ports are connected, please use `buffer_and_port()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer(&self, index: usize) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(&b.0))
    }

    /// Get a reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port(&self, index: usize) -> Option<(AtomicRef<MonoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow(&b.0), b.1))
    }
}

/// Mono audio output buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do ***NOT*** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct MonoProcBuffersMut<T: Num + Copy + Clone> {
    buffers: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<T>>>, usize)>,
}

impl<T: Num + Copy + Clone> MonoProcBuffersMut<T> {
    pub(crate) fn new(buffers: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<T>>>, usize)>) -> Self {
        Self { buffers }
    }

    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers in this list is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// ports are connected, please use `buffer_and_port()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer(&self, index: usize) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(&b.0))
    }

    /// Get a mutable reference to a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// port are connected, please use `buffer_and_port_mut()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_mut(&mut self, index: usize) -> Option<AtomicRefMut<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow_mut(&b.0))
    }

    /// Get a reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port(&self, index: usize) -> Option<(AtomicRef<MonoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow(&b.0), b.1))
    }

    /// Get a mutable reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer_mut()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port_mut(
        &mut self,
        index: usize,
    ) -> Option<(AtomicRefMut<MonoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow_mut(&b.0), b.1))
    }
}

/// Stereo audio input buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do ***NOT*** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct StereoProcBuffers<T: Num + Copy + Clone> {
    buffers: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<T>>>, usize)>,
}

impl<T: Num + Copy + Clone> StereoProcBuffers<T> {
    pub(crate) fn new(buffers: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<T>>>, usize)>) -> Self {
        Self { buffers }
    }

    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers in this list is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// ports are connected, please use `buffer_and_port()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer(&self, index: usize) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(&b.0))
    }

    /// Get a reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port(
        &self,
        index: usize,
    ) -> Option<(AtomicRef<StereoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow(&b.0), b.1))
    }
}

/// Stereo audio output buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do ***NOT*** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct StereoProcBuffersMut<T: Num + Copy + Clone> {
    buffers: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<T>>>, usize)>,
}

impl<T: Num + Copy + Clone> StereoProcBuffersMut<T> {
    pub(crate) fn new(buffers: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<T>>>, usize)>) -> Self {
        Self { buffers }
    }

    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers in this list is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// ports are connected, please use `buffer_and_port()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer(&self, index: usize) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(&b.0))
    }

    /// Get a mutable reference to a specific buffer in this list of buffers.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port. If your node cares about which specific
    /// port are connected, please use `buffer_and_port_mut()` instead. There
    /// will always be only one buffer per port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_mut(&mut self, index: usize) -> Option<AtomicRefMut<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow_mut(&b.0))
    }

    /// Get a reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port(
        &self,
        index: usize,
    ) -> Option<(AtomicRef<StereoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow(&b.0), b.1))
    }

    /// Get a mutable reference to a specific buffer in this list of buffers, while also
    /// returning the index of the port this buffer is assigned to. Use this instead of
    /// `buffer_mut()` if your node cares about which specific ports are connected. There
    /// will always be only one buffer per port.
    ///
    /// Note, `index` is the index that this buffer appears in the list. It is
    /// ***NOT*** the index of the port.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do ***NOT*** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn buffer_and_port_mut(
        &mut self,
        index: usize,
    ) -> Option<(AtomicRefMut<StereoBlockBuffer<T>>, usize)> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| (AtomicRefCell::borrow_mut(&b.0), b.1))
    }
}
