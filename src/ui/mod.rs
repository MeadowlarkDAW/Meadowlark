//! The UI is implemented with the [`VIZIA`] GUI library.
//!
//! [`VIZIA`]: https://github.com/vizia/vizia
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{error::Error, time::Duration};
use vizia::prelude::*;

mod icon;

const MEADOWLARK_ICON_FONT: &[u8] = include_bytes!("resources/fonts/meadowlark-icons.ttf");
const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");
const MIN_SANS_REGULAR: &[u8] = include_bytes!("resources/fonts/MinSans-Regular.otf");
const FIRA_CODE: &[u8] = include_bytes!("resources/fonts/FiraCode-Regular.ttf");

static POLL_TIMER_INTERVAL: Duration = Duration::from_millis(16);

pub struct StateSystem {}

impl StateSystem {
    fn new() -> Self {
        Self {}
    }

    fn poll_engine(&mut self) {
    }
}

impl Model for StateSystem {
    // Update the program layer here
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|program_event, _| match program_event {
            UiEvent::PollEngine => {
                self.poll_engine();
            }
        });
    }
}

#[derive(Debug, Lens, Clone)]
pub struct BoundUiState {}

impl Model for BoundUiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    PollEngine,
}

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    let icon = vizia::image::open("./assets/branding/meadowlark-logo-64.png")?;
    let icon_width = icon.width();
    let icon_height = icon.height();

    let run_poll_timer = Arc::new(AtomicBool::new(true));
    let run_poll_timer_clone = Arc::clone(&run_poll_timer);

    let app = Application::new(move |cx| {
        cx.add_font_mem("meadowlark-icons", MEADOWLARK_ICON_FONT);
        cx.add_font_mem("min-sans-medium", MIN_SANS_MEDIUM);
        cx.add_font_mem("min-sans-regular", MIN_SANS_REGULAR);
        cx.add_font_mem("fira-code", FIRA_CODE);

        cx.add_stylesheet("src/ui/resources/themes/default.css")
            .expect("Failed to find default stylesheet");

        StateSystem::new().build(cx);

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
    .background_color(Color::black())
    .ignore_default_theme();

    app.run();

    run_poll_timer.store(false, Ordering::Relaxed);

    Ok(())
}
