use glib::clone;
use gtk::gio::SimpleAction;
use gtk::glib::{self, VariantTy};
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

mod browser_panel;

use crate::ui::AppWidgets;

pub fn connect_actions(app: &Application, state_system: StateSystem) {
    let state_system = Rc::new(RefCell::new(state_system));

    let action_toggle_browser_panel =
        SimpleAction::new("toggle_browser_panel", Some(VariantTy::BOOLEAN));
    action_toggle_browser_panel.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().toggle_browser_panel(parameter.unwrap().get::<bool>().unwrap());
        }),
    );
    app.add_action(&action_toggle_browser_panel);
}

pub struct StateSystem {
    state: AppState,
    widgets: AppWidgets,
}

impl StateSystem {
    pub fn new(widgets: AppWidgets) -> Self {
        Self { state: AppState::new(), widgets }
    }

    pub fn toggle_browser_panel(&mut self, shown: bool) {
        self.state.browser_panel_shown = shown;
        self.widgets.browser_panel.toggle_shown(shown);
    }
}

pub struct AppState {
    pub project: ProjectSaveState,

    pub browser_panel_shown: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self { project: ProjectSaveState::new(), browser_panel_shown: true }
    }
}

pub struct ProjectSaveState {}

impl ProjectSaveState {
    pub fn new() -> Self {
        Self {}
    }
}
