use gtk::gio::SimpleAction;
use gtk::glib::{self, clone, Continue, VariantTy};
use gtk::prelude::*;
use gtk::Application;
use pcm_loader::ResampleQuality;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

pub mod app_message;
pub mod browser_panel;
pub mod engine_handle;

use crate::backend::resource_loader::PcmKey;
use crate::ui::AppWidgets;

use self::app_message::AppMessage;
use self::browser_panel::BrowserPanelState;
use self::engine_handle::EngineHandle;

static ENGINE_POLL_INTERVAL: Duration = Duration::from_millis(1);

pub fn connect_actions(
    app: &Application,
    state_system: StateSystem,
    app_msg_rx: glib::Receiver<AppMessage>,
) {
    let state_system = Rc::new(RefCell::new(state_system));

    app_msg_rx.attach(
        None,
        clone!(@weak state_system => @default-return Continue(false),
            move |app_msg| {
                state_system.borrow_mut().on_app_message(app_msg);
                Continue(true)
            }
        ),
    );

    let action_set_browser_panel_shown =
        SimpleAction::new("set_browser_panel_shown", Some(VariantTy::BOOLEAN));
    action_set_browser_panel_shown.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().on_set_browser_panel_shown(parameter.unwrap().get::<bool>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_panel_shown);

    let action_set_browser_folder =
        SimpleAction::new("set_browser_folder", Some(VariantTy::UINT64));
    action_set_browser_folder.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().on_set_browser_folder(parameter.unwrap().get::<u64>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_folder);

    let action_browser_item_selected =
        SimpleAction::new("browser_item_selected", Some(VariantTy::UINT32));
    action_browser_item_selected.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().on_browser_item_selected(parameter.unwrap().get::<u32>().unwrap());
        }),
    );
    app.add_action(&action_browser_item_selected);
}

pub struct StateSystem {
    state: AppState,
    widgets: AppWidgets,
    engine_handle: EngineHandle,

    app_msg_tx: glib::Sender<AppMessage>,

    poll_timer_thread_handle: Option<std::thread::JoinHandle<()>>,
    run_poll_timer_thread: Arc<AtomicBool>,
}

impl StateSystem {
    pub fn new(state: AppState, widgets: AppWidgets, app_msg_tx: glib::Sender<AppMessage>) -> Self {
        // Set-up the timer to poll the backend engine periodically.
        let run_poll_timer_thread = Arc::new(AtomicBool::new(true));
        let run = Arc::clone(&run_poll_timer_thread);
        let app_msg_tx_clone = app_msg_tx.clone();
        let poll_timer_thread_handle = std::thread::spawn(move || {
            while run.load(Ordering::Relaxed) {
                app_msg_tx_clone.send(AppMessage::PollEngineTimer).unwrap();
                std::thread::sleep(ENGINE_POLL_INTERVAL);
            }
        });

        let engine_handle = EngineHandle::new();

        let mut new_self = Self {
            state,
            widgets,
            engine_handle,
            app_msg_tx,
            poll_timer_thread_handle: Some(poll_timer_thread_handle),
            run_poll_timer_thread,
        };

        new_self.on_refresh_browser_folder_tree();

        new_self
    }

    pub fn on_app_message(&mut self, app_msg: AppMessage) {
        match app_msg {
            AppMessage::PollEngineTimer => self.on_poll_engine(),
            AppMessage::BrowserPanelFolderTreeRefreshed {
                category,
                folder_tree_model,
                next_entry_id,
            } => {
                if let Some(new_model) = self.state.browser_panel.on_folder_tree_refreshed(
                    category,
                    folder_tree_model,
                    next_entry_id,
                ) {
                    self.widgets.browser_panel.set_folder_tree_model(category, new_model);
                }
            }
            AppMessage::BrowserPanelFileListRefreshed { file_scan_id, file_list_pre_model } => {
                if let Some(new_model) = self
                    .state
                    .browser_panel
                    .on_file_list_refreshed(file_scan_id, file_list_pre_model)
                {
                    self.widgets.browser_panel.set_file_list_model(new_model);
                }
            }
        }
    }

    pub fn on_poll_engine(&mut self) {
        self.engine_handle.on_poll_engine();
    }

    pub fn on_set_browser_panel_shown(&mut self, shown: bool) {
        self.state.browser_panel.shown = shown;
        self.widgets.browser_panel.toggle_shown(shown);
    }

    pub fn on_set_browser_folder(&mut self, id: u64) {
        let do_clear_item_list = self.state.browser_panel.set_browser_folder(id, &self.app_msg_tx);
        if do_clear_item_list {
            self.widgets.browser_panel.clear_file_list();
        }
    }

    pub fn on_browser_item_selected(&mut self, index: u32) {
        if let Some(path) = self.state.browser_panel.on_browser_item_selected(index) {
            self.widgets.browser_panel.set_file_list_item_selected(index);

            if let Some(activated_state) = &mut self.engine_handle.activated_state {
                let pcm_key = PcmKey {
                    path,
                    resample_to_project_sr: true,
                    resample_quality: ResampleQuality::Linear,
                };
                match activated_state.resource_loader.try_load(&pcm_key) {
                    Ok(pcm) => {
                        activated_state.sample_browser_plug_handle.play_sample(pcm);
                    }
                    Err(e) => log::error!("{}", e),
                }
            }
        }
    }

    pub fn on_refresh_browser_folder_tree(&mut self) {
        let do_clear_item_list = self.state.browser_panel.refresh_folder_tree(&self.app_msg_tx);
        if do_clear_item_list {
            self.widgets
                .browser_panel
                .clear_folder_tree(self.state.browser_panel.selected_category);
            self.widgets.browser_panel.clear_file_list();
        }
    }
}

impl Drop for StateSystem {
    fn drop(&mut self) {
        self.run_poll_timer_thread.store(false, Ordering::Relaxed);
        if let Some(handle) = self.poll_timer_thread_handle.take() {
            if handle.join().is_err() {
                log::error!("Failed to join poll timer thread");
            }
        }
    }
}

pub struct AppState {
    pub project: ProjectSaveState,

    pub browser_panel: BrowserPanelState,
}

impl AppState {
    pub fn new() -> Self {
        Self { project: ProjectSaveState::new(), browser_panel: BrowserPanelState::new() }
    }
}

pub struct ProjectSaveState {}

impl ProjectSaveState {
    pub fn new() -> Self {
        Self {}
    }
}
