use ringbuf::Producer;
use rusty_daw_io::{
    FatalErrorHandler, FatalStreamError, ProcessInfo, RtProcessHandler, StreamInfo,
};

use super::rt_state::RtState;

pub struct MainRtHandler {
    state: RtState,
    stream_info: Option<StreamInfo>,
}

impl MainRtHandler {
    pub fn new(state: RtState) -> Self {
        Self {
            state,
            stream_info: None,
        }
    }
}

impl RtProcessHandler for MainRtHandler {
    fn init(&mut self, stream_info: &StreamInfo) {}

    fn process(&mut self, proc_info: ProcessInfo) {
        // Process all new messages from UI
        self.state.sync();

        // TODO: Audio graph stuffs
    }
}

pub struct MainFatalErrorHandler {
    error_signal_tx: Producer<FatalStreamError>,
}

impl MainFatalErrorHandler {
    pub fn new(error_signal_tx: Producer<FatalStreamError>) -> Self {
        Self { error_signal_tx }
    }
}

impl FatalErrorHandler for MainFatalErrorHandler {
    fn fatal_stream_error(mut self, error: FatalStreamError) {
        self.error_signal_tx.push(error).unwrap();
    }
}
