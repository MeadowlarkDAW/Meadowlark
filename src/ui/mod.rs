use image::GenericImageView;
use vizia::*;

use crate::state::{AppEvent, ProjectSaveState, StateSystem};

pub mod icons;

pub mod views;
pub use views::*;

pub mod panels;
pub use panels::*;

const MEADOWLARK_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark.ttf");

pub fn run() -> Result<(), String> {
    let icon = image::open("./assets/branding/meadowlark-logo-64.png").unwrap();

    let window_description = WindowDescription::new()
        .with_title("Meadowlark")
        .with_inner_size(1280, 720)
        .with_icon(icon.to_bytes(), icon.width(), icon.height());

    let app = Application::new(window_description, |cx| {
        let project_save_state = Box::new(ProjectSaveState::test());
        let mut state_system = StateSystem::new();
        state_system.load_project(&project_save_state);

        state_system.build(cx);

        cx.add_font_mem("meadowlark", MEADOWLARK_FONT);

        cx.add_stylesheet("src/ui/resources/themes/default_theme.css");

        VStack::new(cx, |cx| {
            top_bar(cx);
            HStack::new(cx, |cx| {
                left_bar(cx);
                browser(cx);
                channels(cx);
                timeline(cx);
            });
            bottom_bar(cx);
        });
    })
    .background_color(Color::rgb(20, 17, 18))
    .ignore_default_styles();

    let proxy = app.get_proxy();

    std::thread::spawn(move || loop {
        proxy.send_event(Event::new(AppEvent::Sync)).expect("Failed to send proxy event");
        std::thread::sleep(std::time::Duration::from_millis(16));
    });

    app.run();

    Ok(())
}
