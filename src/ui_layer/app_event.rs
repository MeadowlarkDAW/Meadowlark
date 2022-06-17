use vizia::prelude::*;

pub enum AppEvent {
    Sync,

    // ----- Channel -----
    SelectChannel(usize),

    // ----- Track -----
    SelectTrack(usize),
    InsertTrack,
    DuplicateSelectedTrack,
    SelectTrackAbove,
    SelectTrackBelow,
    MoveSelectedTrackUp,
    MoveSelectedTrackDown,
    DeleteSelectedTrack,
}
