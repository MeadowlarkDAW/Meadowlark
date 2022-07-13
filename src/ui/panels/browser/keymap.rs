use vizia::prelude::*;

use super::BrowserEvent;

pub fn browser_keymap(cx: &mut Context) {
    Keymap::from(vec![
        // ArrowDown => Select next directory item.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowDown),
            KeymapEntry::new(BrowserEvent::SelectNext, |cx| {
                cx.emit(BrowserEvent::SelectNext);
                cx.emit(BrowserEvent::PlaySelected);
            }),
        ),
        // ArrowUp => Select previous directory item.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowUp),
            KeymapEntry::new(BrowserEvent::SelectPrev, |cx| {
                cx.emit(BrowserEvent::SelectPrev);
                cx.emit(BrowserEvent::PlaySelected);
            }),
        ),
        // Space => Open/Close the selected directory.
        (
            KeyChord::new(Modifiers::empty(), Code::Space),
            KeymapEntry::new(BrowserEvent::ToggleOpen, |cx| {
                cx.emit(BrowserEvent::ToggleOpen);
            }),
        ),
        // Space => Play the selected browser sample.
        (
            KeyChord::new(Modifiers::empty(), Code::Space),
            KeymapEntry::new(BrowserEvent::PlaySelected, |cx| {
                cx.emit(BrowserEvent::PlaySelected);
            }),
        ),
        // ArrowRight => Play the selected browser sample.
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowRight),
            KeymapEntry::new(BrowserEvent::PlaySelected, |cx| {
                cx.emit(BrowserEvent::PlaySelected);
            }),
        ),
        // ArrowLeft => Stop playing a sample
        (
            KeyChord::new(Modifiers::empty(), Code::ArrowLeft),
            KeymapEntry::new(BrowserEvent::StopSelected, |cx| {
                cx.emit(BrowserEvent::StopSelected);
            }),
        ),
    ])
    .build(cx);
}
