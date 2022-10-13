//! A button that emits a signal when the mouse button is pressed, as
//! apposed to GTK's built-in button widget which emits a signal when
//! the mouse button is released.

mod imp;

use glib::Object;
use gtk::glib;
use gtk::prelude::*;

glib::wrapper! {
    pub struct PressButton(ObjectSubclass<imp::PressButton>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PressButton {
    pub fn new(child: &impl IsA<gtk::Widget>) -> Self {
        Object::new(&[("child", &child.to_value()), ("index", &u32::MAX.to_value())])
            .expect("Failed to create `PressButton`.")
    }

    pub fn child(&self) -> Option<gtk::Widget> {
        self.property::<Option<gtk::Widget>>("child")
    }
}
