//! The UI is implemented with the [`VIZIA`] GUI library.
//!
//! [`VIZIA`]: https://github.com/vizia/vizia
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{error::Error, time::Duration};
use vizia::prelude::*;

use crate::state_system::working_state::timeline_view_state::TimelineViewState;
use crate::state_system::{AppAction, StateSystem};
use crate::ui::panels::{bottom_bar, browser_panel, side_tab_bar, timeline_panel, top_bar};

pub mod generic_views;
pub mod panels;

const MEADOWLARK_ICON_FONT: &[u8] = include_bytes!("resources/icons/meadowlark-icons.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("resources/fonts/Inter-Medium.ttf");
//const INTER_BOLD: &[u8] = include_bytes!("resources/fonts/Inter-Bold.ttf");
const FIRA_CODE: &[u8] = include_bytes!("resources/fonts/FiraCode-Regular.ttf");

static ENGINE_POLL_TIMER_INTERVAL: Duration = Duration::from_millis(4);

pub fn run_ui() -> Result<(), Box<dyn Error>> {
    let icon = vizia::image::open("src/ui/resources/icons/meadowlark-logo-256.png")?;
    let icon_width = icon.width();
    let icon_height = icon.height();

    let run_poll_timer = Arc::new(AtomicBool::new(true));
    let run_poll_timer_clone = Arc::clone(&run_poll_timer);

    let app = Application::new(move |cx| {
        cx.add_fonts_mem(&[INTER_MEDIUM, FIRA_CODE, MEADOWLARK_ICON_FONT]);

        cx.add_stylesheet("src/ui/resources/themes/default.css")
            .expect("Failed to find default stylesheet");

        let shared_timeline_view_state = Rc::new(RefCell::new(TimelineViewState::new()));

        StateSystem::new(Rc::clone(&shared_timeline_view_state)).build(cx);

        VStack::new(cx, |cx| {
            top_bar::top_bar(cx);

            HStack::new(cx, |cx| {
                side_tab_bar::side_tab_bar(cx);
                browser_panel::browser_panel(cx);

                timeline_panel::timeline_panel(cx, shared_timeline_view_state);
            })
            .col_between(Pixels(1.0))
            .width(Stretch(2.0));

            bottom_bar::bottom_bar(cx);
        })
        .background_color(Color::from("#171717"))
        .row_between(Pixels(1.0));

        // Set-up the timer to poll the backend engine periodically.
        let run_poll_timer_clone = Arc::clone(&run_poll_timer_clone);
        cx.spawn(move |cx| {
            while run_poll_timer_clone.load(Ordering::Relaxed) {
                cx.emit(AppAction::_PollEngine).unwrap();
                std::thread::sleep(ENGINE_POLL_TIMER_INTERVAL);
            }
        });
    })
    .title("Meadowlark")
    .inner_size((1280, 720))
    .icon(icon.into_bytes(), icon_width, icon_height)
    .background_color(Color::black())
    .vsync(true)
    .ignore_default_theme();

    app.run();

    run_poll_timer.store(false, Ordering::Relaxed);

    Ok(())
}
