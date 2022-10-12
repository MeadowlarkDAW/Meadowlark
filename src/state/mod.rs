use gtk::gio::SimpleAction;
use gtk::glib::{self, clone, Continue, MainContext, VariantTy, PRIORITY_DEFAULT};
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

pub mod app_message;
pub mod browser_panel;

use crate::ui::AppWidgets;

use self::app_message::AppMessage;
use self::browser_panel::{BrowserCategory, BrowserPanelState};

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
            state_system.borrow_mut().set_browser_panel_shown(parameter.unwrap().get::<bool>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_panel_shown);

    let action_set_browser_folder =
        SimpleAction::new("set_browser_folder", Some(VariantTy::UINT64));
    action_set_browser_folder.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().set_browser_folder(parameter.unwrap().get::<u64>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_folder);
}

pub struct StateSystem {
    state: AppState,
    widgets: AppWidgets,

    app_msg_tx: glib::Sender<AppMessage>,
}

impl StateSystem {
    pub fn new(state: AppState, widgets: AppWidgets, app_msg_tx: glib::Sender<AppMessage>) -> Self {
        let mut new_self = Self { state, widgets, app_msg_tx };

        new_self.refresh_browser_folder_tree();

        new_self
    }

    pub fn on_app_message(&mut self, app_msg: AppMessage) {
        match app_msg {
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
            AppMessage::BrowserPanelFileListRefreshed {
                file_scan_id,
                file_list_pre_model,
                file_id_to_path,
                next_entry_id,
            } => {
                if let Some(new_model) = self.state.browser_panel.on_file_list_refreshed(
                    file_scan_id,
                    file_list_pre_model,
                    file_id_to_path,
                    next_entry_id,
                ) {
                    self.widgets.browser_panel.set_file_list_model(new_model);
                }
            }
        }
    }

    pub fn set_browser_panel_shown(&mut self, shown: bool) {
        self.state.browser_panel.shown = shown;
        self.widgets.browser_panel.toggle_shown(shown);
    }

    pub fn set_browser_folder(&mut self, id: u64) {
        let do_clear_item_list = self.state.browser_panel.set_browser_folder(id, &self.app_msg_tx);
        if do_clear_item_list {
            self.widgets.browser_panel.clear_file_list();
        }
    }

    pub fn refresh_browser_folder_tree(&mut self) {
        let do_clear_item_list = self.state.browser_panel.refresh_folder_tree(&self.app_msg_tx);
        if do_clear_item_list {
            self.widgets
                .browser_panel
                .clear_folder_tree(self.state.browser_panel.selected_category);
            self.widgets.browser_panel.clear_file_list();
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
