//! The UI is implemented with the [`VIZIA`] GUI library.
//!
//! [`VIZIA`]: https://github.com/vizia/vizia
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{error::Error, time::Duration};
use vizia::prelude::*;

pub mod icons;

pub mod state;
pub use state::*;

pub mod views;
pub use views::*;

pub mod panels;
pub use panels::*;

const MEADOWLARK_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark.ttf");
const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");

static POLL_TIMER_INTERVAL: Duration = Duration::from_millis(16);

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    let icon = vizia::image::open("./assets/branding/meadowlark-logo-64.png")?;
    let icon_width = icon.width();
    let icon_height = icon.height();

    let run_poll_timer = Arc::new(AtomicBool::new(true));
    let run_poll_timer_clone = Arc::clone(&run_poll_timer);

    let app = Application::new(move |cx| {
        cx.add_font_mem("meadowlark", MEADOWLARK_FONT);
        cx.add_font_mem("min-sans-medium", MIN_SANS_MEDIUM);

        cx.add_stylesheet("src/ui/resources/themes/default_theme/default_theme.css")
            .expect("Failed to find default stylesheet");
        cx.add_stylesheet("src/ui/resources/themes/default_theme/channel_rack.css")
            .expect("Failed to find default stylesheet");
        cx.add_stylesheet("src/ui/resources/themes/default_theme/top_bar.css")
            .expect("Failed to find default stylesheet");
        cx.add_stylesheet("src/ui/resources/themes/default_theme/bottom_bar.css")
            .expect("Failed to find default stylesheet");
        cx.add_stylesheet("src/ui/resources/themes/default_theme/timeline.css")
            .expect("Failed to find default stylesheet");
        cx.add_stylesheet("src/ui/resources/themes/default_theme/browser.css")
            .expect("Failed to find default stylesheet");

        UiData::new().unwrap().build(cx);

        VStack::new(cx, |cx| {
            // TODO - Move to menu bar
            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| {
                        cx.emit(UiEvent::SaveProject);
                    },
                    |cx| Label::new(cx, "SAVE"),
                )
                .width(Pixels(100.0));

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(UiEvent::LoadProject);
                    },
                    |cx| Label::new(cx, "LOAD"),
                )
                .width(Pixels(100.0));
                Label::new(cx, "File").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "Edit").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "View").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "Help").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
            })
            .class("menu_bar");
            top_bar(cx);
            HStack::new(cx, |cx| {
                browser(cx);
                channels(cx);
                VStack::new(cx, |cx| {
                    timeline(cx);
                    piano_roll(cx);
                })
                .overflow(Overflow::Hidden)
                .class("main")
                .toggle_class(
                    "hidden",
                    UiData::state.then(UiState::panels.then(PanelState::hide_piano_roll)),
                );
            })
            .col_between(Pixels(1.0));
            bottom_bar(cx);
        })
        .background_color(Color::from("#0A0A0A"))
        .row_between(Pixels(1.0));

        let run_poll_timer_clone = Arc::clone(&run_poll_timer_clone);
        cx.spawn(move |cx| {
            while run_poll_timer_clone.load(Ordering::Relaxed) {
                cx.emit(UiEvent::PollEngine).unwrap();
                std::thread::sleep(POLL_TIMER_INTERVAL);
            }
        });
    })
    .title("Meadowlark")
    .inner_size((1280, 720))
    .icon(icon.into_bytes(), icon_width, icon_height)
    //.background_color(Color::rgb(20, 17, 18))
    .ignore_default_theme();

    app.run();

    run_poll_timer.store(false, Ordering::Relaxed);

    Ok(())
}
