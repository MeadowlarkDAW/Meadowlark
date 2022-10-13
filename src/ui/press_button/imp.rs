//! A button that emits a signal when the mouse button is pressed, as
//! apposed to GTK's built-in button widget which emits a signal when
//! the mouse button is released.

use glib::subclass::Signal;
use glib::{clone, ParamSpec, ParamSpecObject, Value};
use gtk::glib::{self, ParamSpecUInt};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::traits::WidgetExt;
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
pub struct PressButton {
    child: RefCell<Option<gtk::Widget>>,

    // TODO: I can't seem to figure out the "Actionable" interface, so
    // for now I'm storing the index value for the browser panel item
    // directly in this widget.
    index: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for PressButton {
    const NAME: &'static str = "MeadowlarkPressButton";
    type Type = super::PressButton;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        // The layout manager determines how child widgets are laid out.
        klass.set_layout_manager_type::<gtk::BinLayout>();

        // Make it look like a GTK button.
        klass.set_css_name("button");

        // Make it appear as a button to accessibility tools.
        klass.set_accessible_role(gtk::AccessibleRole::Button);
    }
}

impl ObjectImpl for PressButton {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecObject::builder("child", glib::types::Type::OBJECT).build(),
                ParamSpecUInt::builder("index").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "child" => {
                let object = value.get::<Option<gtk::Widget>>().unwrap();
                let mut child = self.child.borrow_mut();

                if let Some(old_child) = child.take() {
                    old_child.unparent();
                }

                if let Some(object) = &object {
                    object.set_parent(&*obj);
                }
                *child = object;
            }
            "index" => {
                let index: u32 = value.get::<u32>().unwrap();
                self.index.set(index);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "child" => self.child.borrow().to_value(),
            "index" => self.index.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "pressed",
                // Types of the values which will be sent to the signal handler
                &[],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }

    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // Connect a gesture to handle clicks.
        let gesture = gtk::GestureClick::new();
        gesture.connect_pressed(clone!(@weak obj => move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            obj.emit_by_name::<()>("pressed", &[]);
        }));
        obj.add_controller(&gesture);
    }

    fn dispose(&self, _obj: &Self::Type) {
        // Child widgets need to be manually unparented in `dispose()`.
        if let Some(child) = self.child.borrow_mut().take() {
            child.unparent();
        }
    }
}

impl WidgetImpl for PressButton {}
