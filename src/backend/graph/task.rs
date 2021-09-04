use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use num_traits::Num;

use super::{AudioGraphNode, MonoBlockBuffer, ProcInfo, StereoBlockBuffer};

pub enum AudioGraphTask {
    Node { node: Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>, proc_buffers: ProcBuffers<f32> },
    // TODO: Delay compensation stuffs.
}

impl AudioGraphTask {
    pub fn node(
        node: Shared<AtomicRefCell<Box<dyn AudioGraphNode>>>,
        mono_audio_in: Vec<Shared<AtomicRefCell<MonoBlockBuffer<f32>>>>,
        mono_audio_out: Vec<Shared<AtomicRefCell<MonoBlockBuffer<f32>>>>,
        stereo_audio_in: Vec<Shared<AtomicRefCell<StereoBlockBuffer<f32>>>>,
        stereo_audio_out: Vec<Shared<AtomicRefCell<StereoBlockBuffer<f32>>>>,
    ) -> Self {
        Self::Node {
            node,
            proc_buffers: ProcBuffers {
                mono_audio_in: MonoProcBuffers { buffers: mono_audio_in },
                mono_audio_out: MonoProcBuffersMut { buffers: mono_audio_out },
                stereo_audio_in: StereoProcBuffers { buffers: stereo_audio_in },
                stereo_audio_out: StereoProcBuffersMut { buffers: stereo_audio_out },
            },
        }
    }
}

/// An abstraction that prepares buffers into a nice format for nodes.
pub struct ProcBuffers<T: Num + Copy + Clone> {
    /// Mono audio input buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected. However, it is up to the node to
    /// communicate with the program on which specific ports are connected/disconnected.
    pub mono_audio_in: MonoProcBuffers<T>,

    /// Mono audio output buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected. However, it is up to the node to
    /// communicate with the program on which specific ports are connected/disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    pub mono_audio_out: MonoProcBuffersMut<T>,

    /// Stereo audio input buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected. However, it is up to the node to
    /// communicate with the program on which specific ports are connected/disconnected.
    pub stereo_audio_in: StereoProcBuffers<T>,

    /// Stereo audio output buffers assigned to this node.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that
    /// case it just means some ports are disconnected. However, it is up to the node to
    /// communicate with the program on which specific ports are connected/disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
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
            (&mut *AtomicRefCell::borrow_mut(b)).clear_frames(frames);
        }

        for b in self.stereo_audio_out.buffers.iter() {
            // This should not panic because the rt thread is the only place these buffers
            // are borrowed.
            //
            // TODO: Use unsafe instead of runtime checking? It would be more efficient,
            // but in theory a bug in the scheduler could try and assign the same buffer
            // twice in the same task or in parallel tasks, so it would be nice to
            // detect if that happens.
            (&mut *AtomicRefCell::borrow_mut(b)).clear_frames(frames);
        }
    }
}

/// Mono audio input buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected. However, it is up to the node to
/// communicate with the program on which specific ports are connected/disconnected.
pub struct MonoProcBuffers<T: Num + Copy + Clone> {
    buffers: Vec<Shared<AtomicRefCell<MonoBlockBuffer<T>>>>,
}

impl<T: Num + Copy + Clone> MonoProcBuffers<T> {
    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(b))
    }

    /// Get the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    #[inline]
    pub fn first(&self) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        self.get(0)
    }
}

/// Mono audio output buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected. However, it is up to the node to
/// communicate with the program on which specific ports are connected/disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do **not** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct MonoProcBuffersMut<T: Num + Copy + Clone> {
    buffers: Vec<Shared<AtomicRefCell<MonoBlockBuffer<T>>>>,
}

impl<T: Num + Copy + Clone> MonoProcBuffersMut<T> {
    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
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
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(b))
    }

    /// Get the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn first(&self) -> Option<AtomicRef<MonoBlockBuffer<T>>> {
        self.get(0)
    }

    /// Get a mutable reference to a specific buffer in this list of buffers.
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
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<AtomicRefMut<MonoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow_mut(b))
    }

    /// Get a mutable reference to the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn first_mut(&mut self) -> Option<AtomicRefMut<MonoBlockBuffer<T>>> {
        self.get_mut(0)
    }
}

/// Stereo audio input buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected. However, it is up to the node to
/// communicate with the program on which specific ports are connected/disconnected.
pub struct StereoProcBuffers<T: Num + Copy + Clone> {
    buffers: Vec<Shared<AtomicRefCell<StereoBlockBuffer<T>>>>,
}

impl<T: Num + Copy + Clone> StereoProcBuffers<T> {
    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that the index
    /// is less than this struct's `len()` method. The compiler should elid the unwrap
    /// in that case.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(b))
    }

    /// Get the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    #[inline]
    pub fn first(&self) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        self.get(0)
    }
}

/// Mono audio output buffers assigned to this node.
///
/// The number of buffers may be less than the number of ports on this node. In that
/// case it just means some ports are disconnected. However, it is up to the node to
/// communicate with the program on which specific ports are connected/disconnected.
///
/// Also please note that the audio output buffers may not be cleared to 0.0. As
/// such, please do **not** read from the audio output buffers, and make sure that
/// all unused audio output buffers are manually cleared by the node. You may
/// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
pub struct StereoProcBuffersMut<T: Num + Copy + Clone> {
    buffers: Vec<Shared<AtomicRefCell<StereoBlockBuffer<T>>>>,
}

impl<T: Num + Copy + Clone> StereoProcBuffersMut<T> {
    /// The total number of buffers in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Returns `true` if the number of buffers is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Get a specific buffer in this list of buffers.
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
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow(b))
    }

    /// Get the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn first(&self) -> Option<AtomicRef<StereoBlockBuffer<T>>> {
        self.get(0)
    }

    /// Get a mutable reference to a specific buffer in this list of buffers.
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
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<AtomicRefMut<StereoBlockBuffer<T>>> {
        // This should not panic because the rt thread is the only place these buffers
        // are borrowed.
        //
        // TODO: Use unsafe instead of runtime checking? It would be more efficient,
        // but in theory a bug in the scheduler could try and assign the same buffer
        // twice in the same task or in parallel tasks, so it would be nice to
        // detect if that happens.
        self.buffers.get(index).map(|b| AtomicRefCell::borrow_mut(b))
    }

    /// Get a mutable reference to the first buffer in this list of buffers.
    ///
    /// Please note that this method borrows an atomic reference, which means that
    /// this is inefficient inside per-sample loops. Please use this method outside
    /// of loops if possible.
    ///
    /// You may safely use `unwrap()` if you have previously checked that this struct
    /// is not empty using `is_empty()` or `len()`. The compiler should elid the unwrap
    /// in that case.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As
    /// such, please do **not** read from the audio output buffers, and make sure that
    /// all unused audio output buffers are manually cleared by the node. You may
    /// use `ProcBuffers::clear_audio_out_buffers()` for convenience.
    #[inline]
    pub fn first_mut(&mut self) -> Option<AtomicRefMut<StereoBlockBuffer<T>>> {
        self.get_mut(0)
    }
}
