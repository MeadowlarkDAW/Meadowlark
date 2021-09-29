use crate::state::{
    event::{ProjectEvent, StateSystemEvent},
    ProjectSaveState, StateSystem,
};

pub mod components;
use components::*;

use tuix::*;

const THEME: &str = include_str!("theme.css");

#[derive(Debug, PartialEq, Clone, Copy)]
enum AppEvent {
    TestSetupSetPan(f32),
}

pub struct App {
    state_system: StateSystem,
}

impl App {
    pub fn new() -> Self {
        Self { state_system: StateSystem::new() }
    }
}

impl Widget for App {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, app: Entity) -> Self::Ret {
        Header::default().build(state, app, |builder| builder);
        Timeline::new().build(state, app, |builder| {
            builder
                //.set_height(Pixels(300.0))
                .set_space(Pixels(2.0))
        });

        app.set_background_color(state, Color::rgb(10, 10, 10))
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {}
}

pub fn run() {
    let project_save_state = Box::new(ProjectSaveState::test());

    let window_description = WindowDescription::new().with_title("Meadowlark");
    let app = Application::new(window_description, |state, window| {
        //state.add_theme(DEFAULT_THEME);
        state.add_theme(THEME);

        //let text_to_speech = TextToSpeach::new().build(state, window, |builder| builder);

        let app = App::new().build(state, window, |builder| builder);

        app.emit(state, StateSystemEvent::Project(ProjectEvent::LoadProject(project_save_state)));
    });

    app.run();
}
