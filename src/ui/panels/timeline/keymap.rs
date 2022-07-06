use crate::ui::UiEvent;
use vizia::prelude::*;

pub fn timeline_keymap(cx: &mut Context) {
    Keymap::from(vec![
        // Delete => Delete the selected lane.
        (
            KeyChord::new(Modifiers::empty(), Code::Delete),
            KeymapEntry::new(UiEvent::DeleteSelectedLanes, |cx| {
                cx.emit(UiEvent::DeleteSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + T => Insert a new lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyT),
            KeymapEntry::new(UiEvent::InsertLane, |cx| {
                cx.emit(UiEvent::InsertLane);
                cx.focus();
            }),
        ),
        // CTRL + D => Duplicates the selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyD),
            KeymapEntry::new(UiEvent::DuplicateSelectedLanes, |cx| {
                cx.emit(UiEvent::DuplicateSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + ArrowUp => Moves the selected lanes up by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowUp),
            KeymapEntry::new(UiEvent::MoveSelectedLanesUp, |cx| {
                cx.emit(UiEvent::MoveSelectedLanesUp);
            }),
        ),
        // CTRL + ArrowDown => Moves the selected lanes down by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowDown),
            KeymapEntry::new(UiEvent::MoveSelectedLanesDown, |cx| {
                cx.emit(UiEvent::MoveSelectedLanesDown);
            }),
        ),
        // ArrowUp => Moves the focus to the lane above the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowUp),
            KeymapEntry::new(UiEvent::SelectLaneAbove, |cx| {
                cx.emit(UiEvent::SelectLaneAbove);
            }),
        ),
        // ArrowDown => Moves the focus to the lane below the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowDown),
            KeymapEntry::new(UiEvent::SelectLaneBelow, |cx| {
                cx.emit(UiEvent::SelectLaneBelow);
            }),
        ),
        // 0 => Toggles the activation state of the currently selected lanes.
        (
            KeyChord::new(Modifiers::empty(), Code::Digit0),
            KeymapEntry::new(UiEvent::ToggleSelectedLaneActivation, |cx| {
                cx.emit(UiEvent::ToggleSelectedLaneActivation);
            }),
        ),
        // CTRL + O => Enables the currently selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyO),
            KeymapEntry::new(UiEvent::ActivateSelectedLanes, |cx| {
                cx.emit(UiEvent::ActivateSelectedLanes);
            }),
        ),
        // SHIFT + O => Disables the currently selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::KeyO),
            KeymapEntry::new(UiEvent::DeactivateSelectedLanes, |cx| {
                cx.emit(UiEvent::DeactivateSelectedLanes);
            }),
        ),
        // W => Zooms in vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyW),
            KeymapEntry::new(UiEvent::ZoomInVertically, |cx| {
                cx.emit(UiEvent::ZoomInVertically);
            }),
        ),
        // S => Zooms out vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyS),
            KeymapEntry::new(UiEvent::ZoomOutVertically, |cx| {
                cx.emit(UiEvent::ZoomOutVertically);
            }),
        ),
        // SHIFT + ArrowUp => Decreases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowUp),
            KeymapEntry::new(UiEvent::DecreaseSelectedLaneHeight, |cx| {
                cx.emit(UiEvent::DecreaseSelectedLaneHeight);
            }),
        ),
        // SHIFT + ArrowDown => Increases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowDown),
            KeymapEntry::new(UiEvent::IncreaseSelectedLaneHeight, |cx| {
                cx.emit(UiEvent::IncreaseSelectedLaneHeight);
            }),
        ),
        // CTRL + A => Selects all lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyA),
            KeymapEntry::new(UiEvent::SelectAllLanes, |cx| {
                cx.emit(UiEvent::SelectAllLanes);
            }),
        ),
    ])
    .build(cx);
}
