use glib::clone;
use gtk::gio::SimpleAction;
use gtk::glib::{self, VariantTy};
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

pub mod browser_panel;

use crate::ui::AppWidgets;

use self::browser_panel::BrowserPanelState;

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
}

pub struct StateSystem {
    state: AppState,
    widgets: AppWidgets,
}

impl StateSystem {
    pub fn new(state: AppState, widgets: AppWidgets) -> Self {
        Self { state, widgets }
    }

    pub fn set_browser_panel_shown(&mut self, shown: bool) {
        self.state.browser_panel.shown = shown;
        self.widgets.browser_panel.toggle_shown(shown);
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
