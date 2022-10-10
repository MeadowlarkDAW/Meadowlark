use std::cell::Cell;

use glib::{ParamSpec, ParamSpecInt, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct BrowserPanelListItem {
    number: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for BrowserPanelListItem {
    const NAME: &'static str = "MeadowlarkBrowserPanelListItem";
    type Type = super::BrowserPanelListItem;
}

impl ObjectImpl for BrowserPanelListItem {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> =
            Lazy::new(|| vec![ParamSpecInt::builder("number").build()]);
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "number" => {
                let input_number = value.get().expect("The value needs to be of type `i32`.");
                self.number.replace(input_number);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "number" => self.number.get().to_value(),
            _ => unimplemented!(),
        }
    }
}
