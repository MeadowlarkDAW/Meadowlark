use crate::ui::AppEvent;
use vizia::prelude::*;

pub fn timeline_keymap(cx: &mut Context) {
    Keymap::from(vec![
        // Delete => Delete the selected lane.
        (
            KeyChord::new(Modifiers::empty(), Code::Delete),
            KeymapEntry::new(AppEvent::DeleteSelectedLanes, |cx| {
                cx.emit(AppEvent::DeleteSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + T => Insert a new lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyT),
            KeymapEntry::new(AppEvent::InsertLane, |cx| {
                cx.emit(AppEvent::InsertLane);
                cx.focus();
            }),
        ),
        // CTRL + D => Duplicates the selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyD),
            KeymapEntry::new(AppEvent::DuplicateSelectedLanes, |cx| {
                cx.emit(AppEvent::DuplicateSelectedLanes);
                cx.focus();
            }),
        ),
        // CTRL + ArrowUp => Moves the selected lanes up by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowUp),
            KeymapEntry::new(AppEvent::MoveSelectedLanesUp, |cx| {
                cx.emit(AppEvent::MoveSelectedLanesUp);
            }),
        ),
        // CTRL + ArrowDown => Moves the selected lanes down by one lane.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowDown),
            KeymapEntry::new(AppEvent::MoveSelectedLanesDown, |cx| {
                cx.emit(AppEvent::MoveSelectedLanesDown);
            }),
        ),
        // ArrowUp => Moves the focus to the lane above the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowUp),
            KeymapEntry::new(AppEvent::SelectLaneAbove, |cx| {
                cx.emit(AppEvent::SelectLaneAbove);
            }),
        ),
        // ArrowDown => Moves the focus to the lane below the currently active lane.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowDown),
            KeymapEntry::new(AppEvent::SelectLaneBelow, |cx| {
                cx.emit(AppEvent::SelectLaneBelow);
            }),
        ),
        // 0 => Toggles the activation state of the currently selected lanes.
        (
            KeyChord::new(Modifiers::empty(), Code::Digit0),
            KeymapEntry::new(AppEvent::ToggleSelectedLaneActivation, |cx| {
                cx.emit(AppEvent::ToggleSelectedLaneActivation);
            }),
        ),
        // CTRL + O => Enables the currently selected lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyO),
            KeymapEntry::new(AppEvent::ActivateSelectedLanes, |cx| {
                cx.emit(AppEvent::ActivateSelectedLanes);
            }),
        ),
        // SHIFT + O => Disables the currently selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::KeyO),
            KeymapEntry::new(AppEvent::DeactivateSelectedLanes, |cx| {
                cx.emit(AppEvent::DeactivateSelectedLanes);
            }),
        ),
        // W => Zooms in vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyW),
            KeymapEntry::new(AppEvent::ZoomInVertically, |cx| {
                cx.emit(AppEvent::ZoomInVertically);
            }),
        ),
        // S => Zooms out vertically.
        (
            KeyChord::new(Modifiers::empty(), Code::KeyS),
            KeymapEntry::new(AppEvent::ZoomOutVertically, |cx| {
                cx.emit(AppEvent::ZoomOutVertically);
            }),
        ),
        // SHIFT + ArrowUp => Decreases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowUp),
            KeymapEntry::new(AppEvent::DecreaseSelectedLaneHeight, |cx| {
                cx.emit(AppEvent::DecreaseSelectedLaneHeight);
            }),
        ),
        // SHIFT + ArrowDown => Increases the size of the selected lanes.
        (
            KeyChord::new(Modifiers::SHIFT, Code::ArrowDown),
            KeymapEntry::new(AppEvent::IncreaseSelectedLaneHeight, |cx| {
                cx.emit(AppEvent::IncreaseSelectedLaneHeight);
            }),
        ),
        // CTRL + A => Selects all lanes.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyA),
            KeymapEntry::new(AppEvent::SelectAllLanes, |cx| {
                cx.emit(AppEvent::SelectAllLanes);
            }),
        ),
    ])
    .build(cx);
}
