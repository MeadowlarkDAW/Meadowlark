use clack_extensions::audio_ports::{HostAudioPortsImplementation, RescanType};
use clack_extensions::gui::{GuiError, GuiSize, HostGuiImplementation};
use clack_extensions::latency::HostLatencyImpl;
use clack_extensions::log::implementation::HostLog;
use clack_extensions::log::LogSeverity;
use clack_extensions::note_ports::{
    HostNotePortsImplementation, NoteDialects, NotePortRescanFlags,
};
use clack_extensions::params::{
    HostParamsImplementation, HostParamsImplementationMainThread, ParamClearFlags, ParamRescanFlags,
};
use clack_extensions::thread_check::host::ThreadCheckImplementation;
use clack_extensions::timer::{HostTimerImpl, TimerError, TimerId};
use meadowlark_plugin_api::HostRequestFlags;

use super::{ClapHostMainThread, ClapHostShared};

impl<'a> HostLog for ClapHostShared<'a> {
    fn log(&self, severity: LogSeverity, message: &str) {
        // TODO: Make sure that the log and print methods don't allocate on the current thread.
        // If they do, then we need to come up with a realtime-safe way to print to the terminal.

        let level = match severity {
            LogSeverity::Debug => log::Level::Debug,
            LogSeverity::Info => log::Level::Info,
            LogSeverity::Warning => log::Level::Warn,
            LogSeverity::Error => log::Level::Error,
            LogSeverity::Fatal => log::Level::Error,
            LogSeverity::HostMisbehaving => log::Level::Error,
            LogSeverity::PluginMisbehaving => log::Level::Error,
        };

        log::log!(level, "{}", self.plugin_log_name.as_str());
        log::log!(level, "{}", message);
    }
}

impl<'a> ThreadCheckImplementation for ClapHostShared<'a> {
    fn is_main_thread(&self) -> bool {
        self.thread_ids.is_main_thread()
    }

    fn is_audio_thread(&self) -> bool {
        self.thread_ids.is_process_thread()
    }
}

impl<'a> HostAudioPortsImplementation for ClapHostMainThread<'a> {
    fn is_rescan_flag_supported(&self, mut flag: RescanType) -> bool {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!("Plugin called clap_host_audio_ports->is_rescan_flag_supported() not in the main thread");
            return false;
        }

        let supported = RescanType::FLAGS
            | RescanType::CHANNEL_COUNT
            | RescanType::PORT_TYPE
            | RescanType::IN_PLACE_PAIR
            | RescanType::LIST
            | RescanType::NAMES;

        flag.remove(supported);
        flag.is_empty()
    }

    fn rescan(&mut self, flags: RescanType) {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!("Plugin called clap_host_audio_ports->rescan() not in the main thread");
            return;
        }

        if !flags.is_empty() {
            // We ignore the `flags` field since we just do a full restart
            // and rescan for every "rescan ports" request anyway.
            self.shared.host_request.request(HostRequestFlags::RESCAN_AUDIO_PORTS);
        }
    }
}

impl<'a> HostNotePortsImplementation for ClapHostMainThread<'a> {
    fn supported_dialects(&self) -> NoteDialects {
        // TODO: Support MIDI2
        NoteDialects::CLAP | NoteDialects::MIDI
    }

    fn rescan(&self, flags: NotePortRescanFlags) {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!("Plugin called clap_host_note_ports->rescan() not in the main thread");
            return;
        }

        if !flags.is_empty() {
            // We ignore the `flags` field since we just do a full restart
            // and rescan for every "rescan ports" request anyway.
            self.shared.host_request.request(HostRequestFlags::RESCAN_NOTE_PORTS);
        }
    }
}

impl<'a> HostParamsImplementation for ClapHostShared<'a> {
    #[inline]
    fn request_flush(&self) {
        self.host_request.request(HostRequestFlags::FLUSH_PARAMS);
    }
}

impl<'a> HostParamsImplementationMainThread for ClapHostMainThread<'a> {
    fn rescan(&mut self, flags: ParamRescanFlags) {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!("Plugin called clap_host_params->rescan() not in the main thread");
            return;
        }

        if !flags.is_empty() {
            // We ignore the `flags` field since we just do a full rescan
            // for every "rescan params" request anyway.
            self.shared.host_request.request(HostRequestFlags::RESCAN_PARAMS);
        }
    }

    fn clear(&mut self, _param_id: u32, _flags: ParamClearFlags) {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!("Plugin called clap_host_params->clear() not in the main thread");
            //return;
        }

        // TODO: we have no modulations or automations to clear yet
    }
}

impl<'a> HostTimerImpl for ClapHostMainThread<'a> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, TimerError> {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!(
                "Plugin called clap_host_timer_support->register_timer() not in the main thread"
            );
            return Err(TimerError::RegisterError);
        }

        if let Ok(timer_id) = self.shared.host_request.register_timer(period_ms) {
            Ok(TimerId(timer_id.0))
        } else {
            Err(TimerError::RegisterError)
        }
    }

    fn unregister_timer(
        &mut self,
        timer_id: clack_extensions::timer::TimerId,
    ) -> Result<(), TimerError> {
        if !self.shared.thread_ids.is_main_thread() {
            log::warn!(
                "Plugin called clap_host_timer_support->unregister_timer() not in the main thread"
            );
            return Err(TimerError::RegisterError);
        }

        self.shared
            .host_request
            .unregister_timer(meadowlark_plugin_api::ext::timer::TimerID(timer_id.0));

        Ok(())
    }
}

impl<'a> HostGuiImplementation for ClapHostShared<'a> {
    fn resize_hints_changed(&self) {
        self.host_request.request(HostRequestFlags::GUI_HINTS_CHANGED);
    }

    fn request_resize(&self, new_size: GuiSize) -> Result<(), GuiError> {
        self.host_request.request_gui_resize(new_size);
        Ok(())
    }

    fn request_show(&self) -> Result<(), GuiError> {
        self.host_request.request(HostRequestFlags::GUI_SHOW);
        Ok(())
    }

    fn request_hide(&self) -> Result<(), GuiError> {
        self.host_request.request(HostRequestFlags::GUI_HIDE);
        Ok(())
    }

    fn closed(&self, was_destroyed: bool) {
        if was_destroyed {
            self.host_request.request(HostRequestFlags::GUI_DESTROYED);
        } else {
            self.host_request.request(HostRequestFlags::GUI_CLOSED);
        }
    }
}

impl<'a> HostLatencyImpl for ClapHostMainThread<'a> {
    fn changed(&mut self) {
        self.shared.host_request.request(HostRequestFlags::RESTART);
    }
}
