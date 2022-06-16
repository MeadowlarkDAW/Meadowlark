use vizia::prelude::*;

pub enum AppEvent {
    Sync,
    SelectChannel(usize),
}
