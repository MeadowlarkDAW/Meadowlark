pub mod components;

use tuix::style::themes::DEFAULT_THEME;
use tuix::*;

use self::components::LevelsMeter;

use crate::backend::BackendState;

const THEME: &str = include_str!("theme.css");

#[derive(Debug, PartialEq, Clone, Copy)]
enum AppEvent {
    TestSetupSetPan(f32),
}

pub struct App {
    backend_state: BackendState,
}

impl App {
    pub fn new(backend_state: BackendState) -> Self {
        Self { backend_state }
    }
}

impl Widget for App {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let row = Row::new().build(state, entity, |builder| {
            builder.set_width(Stretch(1.0)).set_height(Stretch(1.0))
        });

        ValueKnob::new("Pan", 0.5, 0.0, 1.0)
            .on_changing(|knob, state, knob_id| {
                state.insert_event(
                    Event::new(AppEvent::TestSetupSetPan(knob.value)).target(knob_id),
                );
            })
            .build(state, row, |builder| {
                builder
                    .set_width(Pixels(50.0))
                    .set_height(Pixels(50.0))
                    .set_space(Stretch(1.0))
            });

        LevelsMeter::new().build(state, row, |builder| {
            builder
                .set_height(Pixels(200.0))
                .set_width(Pixels(50.0))
                .set_space(Stretch(1.0))
                .set_background_color(Color::rgb(50, 50, 50))
        });

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(app_event) = event.message.downcast::<AppEvent>() {
            match app_event {
                AppEvent::TestSetupSetPan(normalized) => self
                    .backend_state
                    .test_setup_pan
                    .as_mut()
                    .unwrap()
                    .pan
                    .set_normalized(*normalized),
            }
        }
    }
}

pub fn run() {
    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let sample_rate = crate::backend::hardware_io::default_sample_rate();

    let (backend_state, rt_shared_state) = BackendState::new(sample_rate);

    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let _stream = crate::backend::rt_thread::run_with_default_output(rt_shared_state);

    let window_description = WindowDescription::new().with_title("Meadowlark");
    let app = Application::new(window_description, |state, window| {
        state.add_theme(DEFAULT_THEME);
        state.add_theme(THEME);

        App::new(backend_state).build(state, window, |builder| builder);
    });

    app.run();
}
