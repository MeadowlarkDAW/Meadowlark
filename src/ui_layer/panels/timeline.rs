use super::tracks::tracks;
use crate::ui_layer::AppEvent;
use vizia::prelude::*;

pub fn timeline(cx: &mut Context) {
    // Keybindings
    Keymap::from(vec![
        // Delete => Delete the selected track.
        (
            KeyChord::new(Modifiers::empty(), Code::Delete),
            KeymapEntry::new(TimelineAction::DeleteSelectedTrack, |cx| {
                cx.emit(AppEvent::DeleteSelectedTrack);
                cx.focus();
            }),
        ),
        // CTRL + T => Insert a new track.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyT),
            KeymapEntry::new(TimelineAction::InsertTrack, |cx| {
                cx.emit(AppEvent::InsertTrack);
                cx.focus();
            }),
        ),
        // CTRL + D => Duplicates the selected track.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyD),
            KeymapEntry::new(TimelineAction::DuplicateSelectedTrack, |cx| {
                cx.emit(AppEvent::DuplicateSelectedTrack);
                cx.focus();
            }),
        ),
        // CTRL + ArrowUp => Moves the selected track up by one track.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowUp),
            KeymapEntry::new(TimelineAction::MoveSelectedTrackUp, |cx| {
                cx.emit(AppEvent::MoveSelectedTrackUp);
            }),
        ),
        // CTRL + ArrowDown => Moves the selected track down by one track.
        (
            KeyChord::new(Modifiers::CTRL, Code::ArrowDown),
            KeymapEntry::new(TimelineAction::MoveSelectedTrackDown, |cx| {
                cx.emit(AppEvent::MoveSelectedTrackDown)
            }),
        ),
        // ArrowUp => Moves the focus to the track above the currently selected track.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowUp),
            KeymapEntry::new(TimelineAction::SelectTrackAbove, |cx| {
                cx.emit(AppEvent::SelectTrackAbove)
            }),
        ),
        // ArrowDown => Moves the focus to the track below the currently selected track.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowDown),
            KeymapEntry::new(TimelineAction::SelectTrackBelow, |cx| {
                cx.emit(AppEvent::SelectTrackBelow)
            }),
        ),
    ])
    .build(cx);

    VStack::new(cx, |cx| {
        // Header
        HStack::new(cx, |cx| {
            Label::new(cx, "TIMELINE").class("small");
        })
        .class("header");

        // Contents
        ScrollView::new(cx, 0.0, 0.0, true, true, |cx| {
            tracks(cx);
        })
        .class("level3");
    })
    .row_between(Pixels(1.0))
    .class("timeline");
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TimelineAction {
    InsertTrack,
    DuplicateSelectedTrack,
    MoveSelectedTrackUp,
    MoveSelectedTrackDown,
    DeleteSelectedTrack,
    SelectTrackAbove,
    SelectTrackBelow,
}
