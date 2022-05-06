use vizia::image::GenericImageView;
use vizia::prelude::*;

use crate::state::{AppEvent, ProjectSaveState, StateSystem};

pub mod icons;

pub mod views;
pub use views::*;

pub mod panels;
pub use panels::*;

const MEADOWLARK_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark.ttf");

pub fn run() -> Result<(), String> {
    let icon = vizia::image::open("./assets/branding/meadowlark-logo-64.png").unwrap();

    let app = Application::new(|cx| {
        let project_save_state = Box::new(ProjectSaveState::test());
        let mut state_system = StateSystem::new();
        state_system.load_project(&project_save_state);

        state_system.build(cx);

        cx.add_font_mem("meadowlark", MEADOWLARK_FONT);

        cx.add_stylesheet("src/ui/resources/themes/default_theme.css");

        PanelState {
            channel_rack_orientation: ChannelRackOrientation::Horizontal,
            hide_patterns: false,
            hide_piano_roll: false,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            top_bar(cx);
            HStack::new(cx, |cx| {
                left_bar(cx);
                browser(cx);
                channels(cx);
                VStack::new(cx, |cx| {
                    timeline(cx);
                    piano_roll(cx);
                });
            })
            .col_between(Pixels(1.0));
            bottom_bar(cx);
        });
    })
    .title("Meadowlark")
    .inner_size((1280, 720))
    //.icon(icon.into_bytes(), icon.width(), icon.height())
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

// TODO - Move this to its own file with other local UI state
#[derive(Lens)]
pub struct PanelState {
    channel_rack_orientation: ChannelRackOrientation,
    hide_patterns: bool,
    hide_piano_roll: bool,
}

pub enum PanelEvent {
    ToggleChannelRackOrientation,
    TogglePatterns,
    ShowPatterns,
    TogglePianoRoll,
}

impl Model for PanelState {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        event.map(|channel_rack_event, _| match channel_rack_event {
            PanelEvent::ToggleChannelRackOrientation => {
                if self.channel_rack_orientation == ChannelRackOrientation::Horizontal {
                    self.channel_rack_orientation = ChannelRackOrientation::Vertical;
                } else {
                    self.channel_rack_orientation = ChannelRackOrientation::Horizontal;
                }
            }

            PanelEvent::TogglePatterns => {
                self.hide_patterns ^= true;
            }

            PanelEvent::ShowPatterns => {
                self.hide_patterns = false;
            }

            PanelEvent::TogglePianoRoll => {
                self.hide_piano_roll ^= true;
            }
        });
    }
}
