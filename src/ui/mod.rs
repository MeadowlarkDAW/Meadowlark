use image::GenericImageView;
use vizia::*;

use crate::state::{AppEvent, ProjectSaveState, StateSystem};

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

        cx.add_stylesheet("src/ui/resources/themes/default_theme.css");

        // UI GOES HERE
    })
    .ignore_default_styles();

    let proxy = app.get_proxy();

    std::thread::spawn(move || loop {
        proxy.send_event(Event::new(AppEvent::Sync)).expect("Failed to send proxy event");
        std::thread::sleep(std::time::Duration::from_millis(16));
    });

    app.run();

    Ok(())
}
