use vizia::prelude::*;

pub enum AppEvent {
    Sync,

    // ----- Channel -----
    SelectChannel(usize),
}
