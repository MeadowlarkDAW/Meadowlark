use bitflags::bitflags;
use clack_extensions::gui::GuiSize;
use std::fmt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use super::ext::timer::TimerID;

bitflags! {
    /// A bitmask of all possible requests to make to the Host's main thread.
    ///
    /// The host is free to not fulfill the request at its own discretion.
    pub struct HostRequestFlags: u32 {
        /// The plugin requested its Audio Processor to be restarted
        const RESTART = 1 << 0;

        /// Should activate the plugin and start processing
        const PROCESS = 1 << 2;

        /// Should call the on_main() callback
        const CALLBACK = 1 << 3;

        /// Should rescan audio ports
        const RESCAN_AUDIO_PORTS = 1 << 4;

        /// Should rescan note ports
        const RESCAN_NOTE_PORTS = 1 << 5;

        /// Should rescan parameters
        const RESCAN_PARAMS = 1 << 6;

        /// Should flush parameter values
        const FLUSH_PARAMS = 1 << 7;

        /// Should resize the GUI
        const GUI_RESIZE = 1 << 8;

        /// Should update GUI resize hints
        const GUI_HINTS_CHANGED = 1 << 9;

        /// Should show the GUI
        const GUI_SHOW = 1 << 10;

        /// Should hide the GUI
        const GUI_HIDE = 1 << 11;

        /// Should register the user closed the floating UI
        const GUI_CLOSED = 1 << 12;

        /// Should register the connection to the UI was lost
        const GUI_DESTROYED = 1 << 13;

        /// The plugin has changed its state and it should be saved again.
        ///
        /// (Note that when a parameter value changes, it is implicit that
        /// the state is dirty and no there is no need to set this flag.)
        const MARK_DIRTY = 1 << 14;

        const TIMER_REQUEST = 1 << 15;
    }
}

/// The receiving end of the Host Request Channel.
///
/// The Host Request Channel is a bitmask-based MPSC communication channel that allows plugins to notify the main
/// thread that certain actions (see [`HostRequestFlags`]) are to be taken.
///
/// This channel **requires** said actions to be idempotent: it does not differentiate
/// between sending one and multiple requests until any of them are received.
///
/// This channel's actions are specific to a specific plugin instance: each plugin instance will
/// have its own channel.
pub struct HostRequestChannelReceiver {
    contents: Arc<HostChannelContents>,
    requested_timers: Arc<Mutex<Vec<HostTimerRequest>>>,
}

impl HostRequestChannelReceiver {
    pub fn new_channel(main_thread_id: std::thread::ThreadId) -> (Self, HostRequestChannelSender) {
        let contents = Arc::new(HostChannelContents::default());
        let requested_timers = Arc::new(Mutex::new(Vec::new()));

        (
            Self { contents: contents.clone(), requested_timers: Arc::clone(&requested_timers) },
            HostRequestChannelSender {
                contents,
                requested_timers,
                main_thread_id,
                next_timer_id: Arc::new(AtomicU32::new(0)),
            },
        )
    }

    /// Returns all the requests that have been made to the channel since the last call to [`fetch_requests`].
    ///
    /// This operation never blocks.
    #[inline]
    pub fn fetch_requests(&self) -> HostRequestFlags {
        HostRequestFlags::from_bits_truncate(
            self.contents.request_flags.swap(HostRequestFlags::empty().bits, Ordering::SeqCst),
        )
    }

    /// Returns the last GUI size that was requested (through a call to [`request_gui_resize`](HostRequestChannelSender::request_gui_resize)).
    ///
    /// This returns [`None`] if no new size has been requested for this plugin yet.
    #[inline]
    pub fn fetch_gui_size_request(&self) -> Option<GuiSize> {
        let size = GuiSize::from_u64(
            self.contents
                .last_gui_size_requested
                .swap(GuiSize { width: u32::MAX, height: u32::MAX }.to_u64(), Ordering::SeqCst),
        );

        match size {
            GuiSize { width: u32::MAX, height: u32::MAX } => None,
            size => Some(size),
        }
    }

    /// Only checks if the plugin has a new timer request. Used to make sure that
    /// a plugin hasn't unregistered a timer before calling its `on_timer()`
    /// method.
    pub fn has_timer_request(&self) -> bool {
        HostRequestFlags::from_bits_truncate(self.contents.request_flags.load(Ordering::SeqCst))
            .contains(HostRequestFlags::TIMER_REQUEST)
    }

    pub fn fetch_timer_requests(&mut self) -> Vec<HostTimerRequest> {
        let mut v = Vec::new();

        // Using a mutex here is realtime-safe because this is only used in the main
        // thread.
        let mut requested_timers = self.requested_timers.lock().unwrap();
        if !requested_timers.is_empty() {
            std::mem::swap(&mut *requested_timers, &mut v)
        }

        v
    }
}

/// The sender end of the Host Request Channel.
///
/// See the [`HostRequestChannelReceiver`] docs for more information about how this works.
///
/// Cloning this sender does not clone the underlying data: all cloned copies will be linked to the
/// same channel.
#[derive(Clone)]
pub struct HostRequestChannelSender {
    contents: Arc<HostChannelContents>,
    requested_timers: Arc<Mutex<Vec<HostTimerRequest>>>,
    main_thread_id: std::thread::ThreadId,
    next_timer_id: Arc<AtomicU32>,
}

impl HostRequestChannelSender {
    pub fn request(&self, flags: HostRequestFlags) {
        self.contents.request_flags.fetch_or(flags.bits, Ordering::SeqCst);
    }

    pub fn request_gui_resize(&self, new_size: GuiSize) {
        self.contents.last_gui_size_requested.store(new_size.to_u64(), Ordering::SeqCst);
        self.request(HostRequestFlags::GUI_RESIZE)
    }

    /// Request the host to register a timer for this plugin.
    ///
    /// This will return an error if not called on the main thread.
    pub fn register_timer(&self, period_ms: u32) -> Result<TimerID, MainThreadError> {
        if std::thread::current().id() == self.main_thread_id {
            let timer_id = TimerID(self.next_timer_id.load(Ordering::SeqCst));
            self.next_timer_id.store(timer_id.0 + 1, Ordering::SeqCst);

            // Using a mutex here is realtime-safe because we only allow this mutex
            // to be used in the main thread.
            let mut requested_timers = self.requested_timers.lock().unwrap();
            requested_timers.push(HostTimerRequest { timer_id, period_ms, register: true });

            self.request(HostRequestFlags::TIMER_REQUEST);

            Ok(timer_id)
        } else {
            Err(MainThreadError)
        }
    }

    /// Request the host to unregister a timer for this plugin.
    ///
    /// This can only be called on the main thread.
    pub fn unregister_timer(&self, timer_id: TimerID) {
        // Using a mutex here is realtime-safe because we only allow this mutex
        // to be used in the main thread.
        if std::thread::current().id() == self.main_thread_id {
            let mut requested_timers = self.requested_timers.lock().unwrap();
            requested_timers.push(HostTimerRequest { timer_id, period_ms: 0, register: false });

            self.request(HostRequestFlags::TIMER_REQUEST);
        }
    }
}

struct HostChannelContents {
    request_flags: AtomicU32,           // HostRequestFlags
    last_gui_size_requested: AtomicU64, // GuiSize, default value (i.e. never requested) = MAX
}

impl Default for HostChannelContents {
    fn default() -> Self {
        Self {
            request_flags: AtomicU32::new(HostRequestFlags::empty().bits),
            last_gui_size_requested: AtomicU64::new(
                GuiSize { width: u32::MAX, height: u32::MAX }.to_u64(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostTimerRequest {
    pub timer_id: TimerID,
    pub period_ms: u32,

    /// `true` = register, `false` = unregister
    pub register: bool,
}

/// This error is returned if a method that is meant to only be called
/// on the main thread is not called on the main thread.
#[derive(Debug, Clone, Copy)]
pub struct MainThreadError;

impl fmt::Display for MainThreadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Method was not called on the main thread")
    }
}

impl std::error::Error for MainThreadError {}
