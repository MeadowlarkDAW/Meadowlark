use std::cell::{Cell, RefCell};

use glib::{ParamSpec, ParamSpecInt, Value};
use gtk::glib::ParamSpecBoolean;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, ListStore};
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct BrowserPanelTreeListItem {
    number: Cell<i32>,
    has_children: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for BrowserPanelTreeListItem {
    const NAME: &'static str = "MeadowlarkBrowserPanelTreeListItem";
    type Type = super::BrowserPanelTreeListItem;
}

impl ObjectImpl for BrowserPanelTreeListItem {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecInt::builder("number").build(),
                ParamSpecBoolean::builder("has_children").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "number" => {
                let input_number = value.get().expect("The value needs to be of type `i32`.");
                self.number.replace(input_number);
            }
            "has_children" => {
                self.has_children.replace(value.get().unwrap());
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "number" => self.number.get().to_value(),
            "has_children" => self.has_children.get().to_value(),
            _ => unimplemented!(),
        }
    }
}
