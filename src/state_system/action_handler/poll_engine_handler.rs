// BIG TODO: Have the entire engine run in a separate process for
// crash protection from buggy plugins.

use dropseed::engine::error::EngineCrashError;
use dropseed::engine::{EngineDeactivatedStatus, OnIdleEvent};
use std::time::Instant;
use vizia::prelude::*;

use crate::backend::engine_handle::GARBAGE_COLLECT_INTERVAL;
use crate::state_system::{EngineHandle, SourceState, WorkingState};
use crate::ui::panels::timeline_panel::TimelineViewEvent;

pub fn poll_engine(
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    let now = Instant::now();
    if now >= engine_handle.next_timer_instant {
        let (mut events, next_timer_instant) = engine_handle.ds_engine.on_timer();
        engine_handle.next_timer_instant = next_timer_instant;

        let mut status = EnginePollStatus::Ok;

        for event in events.drain(..) {
            match on_engine_event(engine_handle, event) {
                EnginePollStatus::Ok => continue,
                s => {
                    status = s;
                    break;
                }
            }
        }

        match status {
            EnginePollStatus::Ok => {
                poll_plugins(cx, source_state, working_state, engine_handle);
            }
            EnginePollStatus::EngineDeactivatedGracefully => {
                log::info!("Engine deactivated gracefully");
            }
            EnginePollStatus::EngineCrashed(error_msg) => {
                log::error!("Engine crashed: {}", error_msg);
            }
        }
    }

    if now >= engine_handle.next_garbage_collect_instant {
        if let Some(activated_handles) = &mut engine_handle.activated_handles {
            activated_handles.resource_loader.collect();
        }

        engine_handle.next_garbage_collect_instant = now + GARBAGE_COLLECT_INTERVAL;
    }
}

fn poll_plugins(
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    // Poll the current position of the playhead if the transport is playing.
    if working_state.transport_playing {
        if let Some(current_project) = &source_state.current_project {
            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                let (new_playhead_frame, playhead_moved) = activated_handles
                    .engine_info
                    .transport_handle
                    .current_playhead_position_frames();
                if playhead_moved {
                    {
                        working_state
                            .shared_timeline_view_state
                            .borrow_mut()
                            .update_playhead_position(
                                new_playhead_frame,
                                &current_project.tempo_map,
                            );
                    }

                    cx.emit_to(
                        working_state.timeline_view_id.unwrap(),
                        TimelineViewEvent::PlayheadMoved,
                    );
                }
            }
        }
    }
}

fn on_engine_event(engine_handle: &mut EngineHandle, event: OnIdleEvent) -> EnginePollStatus {
    match event {
        // The plugin's parameters have been modified via the plugin's custom
        // GUI.
        //
        // Only the parameters which have changed will be returned in this
        // field.
        OnIdleEvent::PluginParamsModified { plugin_id, modified_params } => {}

        // The plugin requested the app to resize its gui to the given size.
        //
        // This event will only be sent if using an embedded window for the
        // plugin GUI.
        OnIdleEvent::PluginRequestedToResizeGui { plugin_id, size } => {}

        // The plugin requested the app to show its GUI.
        //
        // This event will only be sent if using an embedded window for the
        // plugin GUI.
        OnIdleEvent::PluginRequestedToShowGui { plugin_id } => {}

        // The plugin requested the app to hide its GUI.
        //
        // Note that hiding the GUI is not the same as destroying the GUI.
        // Hiding only hides the window content, it does not free the GUI's
        // resources.  Yet it may be a good idea to stop painting timers
        // when a plugin GUI is hidden.
        //
        // This event will only be sent if using an embedded window for the
        // plugin GUI.
        OnIdleEvent::PluginRequestedToHideGui { plugin_id } => {}

        // Sent when the plugin closed its own GUI by its own means. UI should
        // be updated accordingly so that the user could open the UI again.
        //
        // If `was_destroyed` is `true`, then the app must call
        // `PluginHostMainThread::destroy_gui()` to acknowledge the gui
        // destruction.
        OnIdleEvent::PluginGuiClosed { plugin_id, was_destroyed } => {}

        // Sent when the plugin changed the resize hint information on how
        // to resize its GUI.
        //
        // This event will only be sent if using an embedded window for the
        // plugin GUI.
        OnIdleEvent::PluginChangedGuiResizeHints { plugin_id, resize_hints } => {}

        // The plugin has updated its list of parameters.
        OnIdleEvent::PluginUpdatedParameterList { plugin_id, status } => {}

        // Sent whenever a plugin becomes activated after being deactivated or
        // when the plugin restarts.
        //
        // Make sure your UI updates the port configuration on this plugin, as
        // well as any custom handles.
        OnIdleEvent::PluginActivated { plugin_id, status } => {}

        // Sent whenever a plugin has been deactivated. When a plugin is
        // deactivated, you cannot access any of its methods until it is
        // reactivated.
        OnIdleEvent::PluginDeactivated { plugin_id, status } => {}

        // Sent whenever the engine has been deactivated, whether gracefully or
        // because of a crash.
        OnIdleEvent::EngineDeactivated(status) => {
            engine_handle.activated_handles = None;
            engine_handle.system_io_stream_handle.on_engine_deactivated();

            match status {
                EngineDeactivatedStatus::DeactivatedGracefully => {
                    return EnginePollStatus::EngineDeactivatedGracefully;
                }
                EngineDeactivatedStatus::EngineCrashed(e) => {
                    return EnginePollStatus::EngineCrashed(e);
                }
            }
        }
    }

    EnginePollStatus::Ok
}

enum EnginePollStatus {
    Ok,
    EngineDeactivatedGracefully,
    EngineCrashed(Box<EngineCrashError>),
}
