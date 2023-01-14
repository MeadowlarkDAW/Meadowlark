use pcm_loader::ResampleQuality;
use vizia::prelude::*;

use crate::resource::PcmKey;
use crate::state_system::{BrowserPanelAction, EngineHandle, SourceState, WorkingState};

pub fn handle_browser_panel_action(
    action: &BrowserPanelAction,
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    match action {
        BrowserPanelAction::SetPanelShown(shown) => {
            source_state.app.browser_panel.panel_shown = *shown;
            working_state.browser_panel_lens.panel_shown = *shown;
        }
        BrowserPanelAction::SelectTab(tab) => {
            source_state.app.browser_panel.current_tab = *tab;
            working_state.browser_panel_lens.current_tab = *tab;
        }
        BrowserPanelAction::SetPanelWidth(width) => {
            let width = width.clamp(170.0, 2000.0);
            source_state.app.browser_panel.panel_width = width;
            working_state.browser_panel_lens.panel_width = width;
        }
        BrowserPanelAction::SetSearchText(text) => {
            working_state.browser_panel_lens.search_text = text.clone();
        }
        BrowserPanelAction::SetVolumeNormalized(volume_normalized) => {
            let volume_normalized = volume_normalized.clamp(0.0, 1.0);

            source_state.app.browser_panel.volume_normalized = volume_normalized;
            working_state.browser_panel_lens.volume.value_normalized = volume_normalized;

            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                engine_handle
                    .ds_engine
                    .plugin_host_mut(&activated_handles.sample_browser_plug_id)
                    .unwrap()
                    .set_param_value(
                        activated_handles.sample_browser_plug_params[0],
                        f64::from(volume_normalized),
                    )
                    .unwrap();
            }
        }
        BrowserPanelAction::SelectEntryByIndex { index, invoked_by_play_btn } => {
            working_state.browser_panel_lens.select_entry_by_index(
                cx,
                *index,
                *invoked_by_play_btn,
            );
        }
        BrowserPanelAction::EnterParentDirectory => {
            working_state.browser_panel_lens.enter_parent_directory();
        }
        BrowserPanelAction::EnterRootDirectory => {
            working_state.browser_panel_lens.enter_root_directory();
        }
        BrowserPanelAction::SetPlaybackOnSelect(val) => {
            source_state.app.browser_panel.playback_on_select = *val;
            working_state.browser_panel_lens.playback_on_select = *val;
        }
        BrowserPanelAction::PlayFile(path) => {
            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                let pcm_key = PcmKey {
                    path: path.clone(),
                    resample_to_project_sr: true,
                    resample_quality: ResampleQuality::Linear,
                };
                match activated_handles.resource_loader.try_load(&pcm_key) {
                    Ok(pcm) => {
                        activated_handles.sample_browser_plug_handle.play_pcm(pcm);
                    }
                    Err(e) => log::error!("{}", e),
                }
            }
        }
        BrowserPanelAction::StopPlayback => {
            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                activated_handles.sample_browser_plug_handle.stop();
            }
        }
        BrowserPanelAction::Refresh => {
            working_state.browser_panel_lens.refresh();
        }
    }
}
