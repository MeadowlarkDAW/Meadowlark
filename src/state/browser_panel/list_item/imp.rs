use std::cell::{Cell, RefCell};

use glib::{ParamSpec, ParamSpecString, ParamSpecUChar, ParamSpecUInt, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

#[repr(u8)]
pub enum BrowserPanelItemType {
    Audio = 0,
}

impl BrowserPanelItemType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Audio),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            BrowserPanelItemType::Audio => 0,
        }
    }
}

#[derive(Default)]
pub struct BrowserPanelListItem {
    index: Cell<u32>,
    item_type: Cell<u8>,
    name: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for BrowserPanelListItem {
    const NAME: &'static str = "MeadowlarkBrowserPanelListItem";
    type Type = super::BrowserPanelListItem;
}

impl ObjectImpl for BrowserPanelListItem {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecUInt::builder("index").build(),
                ParamSpecUChar::builder("item-type").build(),
                ParamSpecString::builder("name").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "index" => {
                let index = value.get().unwrap();
                self.index.replace(index);
            }
            "item-type" => {
                let item_type = value.get().unwrap();
                self.item_type.replace(item_type);
            }
            "name" => {
                let name = value.get().unwrap();
                *self.name.borrow_mut() = name;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "index" => self.index.get().to_value(),
            "item-type" => self.item_type.get().to_value(),
            "name" => self.name.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}
