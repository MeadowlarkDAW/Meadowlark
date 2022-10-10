use gtk::glib;
use gtk::subclass::prelude::*;

#[derive(Default)]
pub struct StateSystem {

}

#[glib::object_subclass]
impl ObjectSubclass for StateSystem {
    const NAME: &'static str = "MeadowlarkStateSystem";
    type Type = super::StateSystem;
    type ParentType = glib::Object;
}