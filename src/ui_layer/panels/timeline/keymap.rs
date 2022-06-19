use crate::program_layer::ProgramEvent;
use vizia::prelude::*;

pub fn timeline_keymap(cx: &mut Context) {
    Keymap::from(vec![
        // Delete => Delete the selected lane.
        (
            KeyChord::new(Modifiers::empty(), Code::Delete),
            KeymapEntry::new(ProgramEvent::DeleteSelectedLanes, |cx| {
                cx.emit(ProgramEvent::DeleteSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + T => Insert a new lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyT),
            KeymapEntry::new(ProgramEvent::InsertLane, |cx| {
                cx.emit(ProgramEvent::InsertLane);
                cx.focus();
            }),
        ),
        // CTRL + D => Duplicates the selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyD),
            KeymapEntry::new(ProgramEvent::DuplicateSelectedLanes, |cx| {
                cx.emit(ProgramEvent::DuplicateSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + ArrowUp => Moves the selected lanes up by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowUp),
            KeymapEntry::new(ProgramEvent::MoveSelectedLanesUp, |cx| {
                cx.emit(ProgramEvent::MoveSelectedLanesUp);
            }),
        ),
        // CTRL + ArrowDown => Moves the selected lanes down by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowDown),
            KeymapEntry::new(ProgramEvent::MoveSelectedLanesDown, |cx| {
                cx.emit(ProgramEvent::MoveSelectedLanesDown);
            }),
        ),
        // ArrowUp => Moves the focus to the lane above the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowUp),
            KeymapEntry::new(ProgramEvent::SelectLaneAbove, |cx| {
                cx.emit(ProgramEvent::SelectLaneAbove);
            }),
        ),
        // ArrowDown => Moves the focus to the lane below the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowDown),
            KeymapEntry::new(ProgramEvent::SelectLaneBelow, |cx| {
                cx.emit(ProgramEvent::SelectLaneBelow);
            }),
        ),
        // 0 => Toggles the activation state of the currently selected lanes.
        (
            KeyChord::new(Modifiers::empty(), Code::Digit0),
            KeymapEntry::new(ProgramEvent::ToggleSelectedLaneActivation, |cx| {
                cx.emit(ProgramEvent::ToggleSelectedLaneActivation);
            }),
        ),
        // CTRL + O => Enables the currently selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyO),
            KeymapEntry::new(ProgramEvent::ActivateSelectedLanes, |cx| {
                cx.emit(ProgramEvent::ActivateSelectedLanes);
            }),
        ),
        // SHIFT + O => Disables the currently selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::KeyO),
            KeymapEntry::new(ProgramEvent::DeactivateSelectedLanes, |cx| {
                cx.emit(ProgramEvent::DeactivateSelectedLanes);
            }),
        ),
        // W => Zooms in vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyW),
            KeymapEntry::new(ProgramEvent::ZoomInVertically, |cx| {
                cx.emit(ProgramEvent::ZoomInVertically);
            }),
        ),
        // S => Zooms out vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyS),
            KeymapEntry::new(ProgramEvent::ZoomOutVertically, |cx| {
                cx.emit(ProgramEvent::ZoomOutVertically);
            }),
        ),
        // SHIFT + ArrowUp => Decreases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowUp),
            KeymapEntry::new(ProgramEvent::DecreaseSelectedLaneHeight, |cx| {
                cx.emit(ProgramEvent::DecreaseSelectedLaneHeight);
            }),
        ),
        // SHIFT + ArrowDown => Increases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowDown),
            KeymapEntry::new(ProgramEvent::IncreaseSelectedLaneHeight, |cx| {
                cx.emit(ProgramEvent::IncreaseSelectedLaneHeight);
            }),
        ),
        // CTRL + A => Selects all lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyA),
            KeymapEntry::new(ProgramEvent::SelectAllLanes, |cx| {
                cx.emit(ProgramEvent::SelectAllLanes);
            }),
        ),
    ])
    .build(cx);
}
