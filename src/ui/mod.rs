use image::GenericImageView;
use vizia::*;

use crate::state::{AppEvent, ProjectSaveState, StateSystem};

mod tempo_controls;
use tempo_controls::tempo_controls;

mod transport_controls;
use transport_controls::transport_controls;

mod timeline_view;
use timeline_view::timeline_view;

mod track_controls;
pub use track_controls::*;

mod track;
pub use track::*;

mod loop_region;
pub use loop_region::*;

mod clip;
pub use clip::*;

mod keymap;
pub use keymap::*;

mod waveform;
pub use waveform::*;

mod timeline_grid;
pub use timeline_grid::*;

pub fn run() -> Result<(), String> {
    let icon = image::open("./assets/branding/meadowlark-logo-32.png").unwrap();

    let window_description = WindowDescription::new()
        .with_title("Meadowlark")
        .with_inner_size(1280, 720)
        .with_icon(icon.to_bytes(), icon.width(), icon.height());

    let app = Application::new(window_description, |cx| {
        let project_save_state = Box::new(ProjectSaveState::test());
        let mut state_system = StateSystem::new();
        state_system.load_project(&project_save_state);

        state_system.build(cx);

        cx.add_stylesheet("src/ui/resources/themes/style.css");

        Keymap::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // Top bar controls
                HStack::new(cx, |cx| {
                    tempo_controls(cx).width(Pixels(300.0));
                    Element::new(cx).class("divider");
                    transport_controls(cx);
                })
                .height(Pixels(70.0))
                .background_color(Color::rgb(63, 57, 59))
                .bottom(Pixels(1.0));

                // Tracks View
                timeline_view(cx);
            })
            .background_color(Color::rgb(10, 10, 10));
        });
    });

    let proxy = app.get_proxy();

    std::thread::spawn(move || loop {
        proxy.send_event(Event::new(AppEvent::Sync)).expect("Failed to send proxy event");
        std::thread::sleep(std::time::Duration::from_millis(16));
    });

    app.run();

    Ok(())
}
