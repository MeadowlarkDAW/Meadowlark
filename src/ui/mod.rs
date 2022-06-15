use vizia::image::GenericImageView;
use vizia::prelude::*;

use crate::state::{AppEvent, ProjectSaveState, StateSystem};

pub mod icons;

pub mod views;
pub use views::*;

pub mod panels;
pub use panels::*;

pub mod app_data;
pub use app_data::*;

pub mod app_event;
pub use app_event::*;

const MEADOWLARK_FONT: &[u8] = include_bytes!("resources/fonts/Meadowlark.ttf");
const MIN_SANS_MEDIUM: &[u8] = include_bytes!("resources/fonts/MinSans-Medium.otf");
// TODO - Add other fonts

pub fn run() -> Result<(), String> {
    let icon = vizia::image::open("./assets/branding/meadowlark-logo-64.png").unwrap();
    let icon_width = icon.width();
    let icon_height = icon.height();

    let app = Application::new(|cx| {
        let project_save_state = Box::new(ProjectSaveState::test());
        let mut state_system = StateSystem::new();
        state_system.load_project(&project_save_state);

        state_system.build(cx);

        cx.add_font_mem("meadowlark", MEADOWLARK_FONT);
        cx.add_font_mem("min-sans-medium", MIN_SANS_MEDIUM);

        cx.add_stylesheet("src/ui/resources/themes/default_theme.css");

        // Dummy AppData for now
        AppData {
            channel_data: vec![
                ChannelData {
                    name: String::from("Master"),
                    selected: false,
                    color: Color::from("#D4D5D5"),
                    subchannels: vec![1, 5],
                },
                ChannelData {
                    name: String::from("Drum Group"),
                    selected: false,
                    color: Color::from("#EDE171"),
                    subchannels: vec![2, 3, 4],
                },
                ChannelData {
                    name: String::from("Kick"),
                    selected: false,
                    color: Color::from("#EDE171"),
                    subchannels: vec![],
                },
                ChannelData {
                    name: String::from("Snare"),
                    selected: true,
                    color: Color::from("#EDE171"),
                    subchannels: vec![],
                },
                ChannelData {
                    name: String::from("Hat"),
                    selected: false,
                    color: Color::from("#EDE171"),
                    subchannels: vec![],
                },
                ChannelData {
                    name: String::from("Spicy Synth"),
                    selected: false,
                    color: Color::from("#EA716C"),
                    subchannels: vec![],
                },
            ],

            pattern_data: vec![PatternData { name: String::from("Drum Group 1"), channel: 1 }],
        }
        .build(cx);

        PanelState {
            channel_rack_orientation: ChannelRackOrientation::Horizontal,
            hide_patterns: false,
            hide_piano_roll: false,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            // TODO - Move to menu bar
            HStack::new(cx, |cx| {
                Label::new(cx, "File").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "Edit").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "View").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
                Label::new(cx, "Help").width(Pixels(50.0)).child_space(Stretch(1.0)).class("small");
            })
            .class("menu_bar");
            top_bar(cx);
            HStack::new(cx, |cx| {
                left_bar(cx);
                browser(cx);
                channels(cx);
                VStack::new(cx, |cx| {
                    timeline(cx);
                    piano_roll(cx);
                })
                .overflow(Overflow::Hidden)
                .class("main")
                .toggle_class("hidden", PanelState::hide_piano_roll);
            })
            .col_between(Pixels(1.0));
            bottom_bar(cx);
        })
        .background_color(Color::from("#0A0A0A"))
        .row_between(Pixels(1.0));
    })
    .title("Meadowlark")
    .inner_size((1280, 720))
    .icon(icon.into_bytes(), icon_width, icon_height)
    //.background_color(Color::rgb(20, 17, 18))
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
