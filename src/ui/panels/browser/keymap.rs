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
        // Enter => Play the selected browser sample.
        (
            KeyChord::new(Modifiers::empty(), Code::Enter),
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
    ])
    .build(cx);
}
