use gtk::gio::SimpleAction;
use gtk::glib::{self, clone, Continue, VariantTy};
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

use super::app_message::AppMessage;
use super::StateSystem;

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

    // ---  Browser Panel  ------------------------------------------------------------------------------

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

    let action_set_browser_playback =
        SimpleAction::new("set_browser_playback", Some(VariantTy::BOOLEAN));
    action_set_browser_playback.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().on_set_browser_playback(parameter.unwrap().get::<bool>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_playback);

    let action_set_browser_playback_volume =
        SimpleAction::new("set_browser_playback_volume", Some(VariantTy::DOUBLE));
    action_set_browser_playback_volume.connect_activate(
        clone!(@strong state_system => move |_action, parameter| {
            state_system.borrow_mut().on_set_browser_playback_volume(parameter.unwrap().get::<f64>().unwrap());
        }),
    );
    app.add_action(&action_set_browser_playback_volume);

    let action_browser_playback_play = SimpleAction::new("browser_playback_play", None);
    action_browser_playback_play.connect_activate(
        clone!(@strong state_system => move |_action, _parameter| {
            state_system.borrow_mut().on_browser_playback_play();
        }),
    );
    app.add_action(&action_browser_playback_play);

    let action_browser_playback_stop = SimpleAction::new("browser_playback_stop", None);
    action_browser_playback_stop.connect_activate(
        clone!(@strong state_system => move |_action, _parameter| {
            state_system.borrow_mut().on_browser_playback_stop();
        }),
    );
    app.add_action(&action_browser_playback_stop);
}
