use glib::clone;
use gtk::gio::SimpleAction;
use gtk::glib::{self, VariantTy};
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

pub mod browser_panel;

use crate::ui::AppWidgets;

use self::browser_panel::{BrowserCategory, BrowserPanelState};

pub fn connect_actions(app: &Application, state_system: StateSystem) {
    let state_system = Rc::new(RefCell::new(state_system));

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
}

impl StateSystem {
    pub fn new(state: AppState, widgets: AppWidgets) -> Self {
        let mut new_self = Self { state, widgets };

        new_self.refresh_browser_folder_tree();

        new_self
    }

    pub fn set_browser_panel_shown(&mut self, shown: bool) {
        self.state.browser_panel.shown = shown;
        self.widgets.browser_panel.toggle_shown(shown);
    }

    pub fn set_browser_folder(&mut self, id: u64) {
        let do_refresh_item_list = self.state.browser_panel.set_browser_folder(id);
        if do_refresh_item_list {
            self.widgets.browser_panel.refresh_item_list(&self.state.browser_panel.file_list_model);
        }
    }

    pub fn refresh_browser_folder_tree(&mut self) {
        let current_category = self.state.browser_panel.selected_category;

        if let Some(new_model) = self.state.browser_panel.refresh_folder_tree() {
            self.widgets.browser_panel.refresh_folder_tree(current_category, new_model);
            self.widgets.browser_panel.refresh_item_list(&self.state.browser_panel.file_list_model);
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
