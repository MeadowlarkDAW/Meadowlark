//use eframe::{egui, epi};

use crate::backend::{ProjectSaveState, ProjectStateInterface};

/*
pub fn run() {
    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let sample_rate = crate::backend::hardware_io::default_sample_rate();

    // TODO: Load project state from file.
    let save_state = ProjectSaveState::test(sample_rate);

    let (mut project_interface, rt_state, load_errors) =
        ProjectStateInterface::new(save_state, sample_rate);

    project_interface.timeline_transport_mut().set_playing(true);

    // TODO: Alert user of any load errors.
    for error in load_errors.iter() {
        log::error!("{:?}", error);
    }

    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let _stream = crate::backend::rt_thread::run_with_default_output(rt_state);

    let app = AppPrototype::new(project_interface);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

struct AppPrototype {
    project_interface: ProjectStateInterface,
}

impl AppPrototype {
    pub fn new(project_interface: ProjectStateInterface) -> Self {
        Self { project_interface }
    }
}

impl epi::App for AppPrototype {
    fn name(&self) -> &str {
        "Meadowlark Prototype"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, world!");
        });
    }
}

*/

pub mod components;
use components::*;

pub mod app_data;
pub use app_data::*;

use tuix::style::themes::DEFAULT_THEME;
use tuix::*;

const THEME: &str = include_str!("theme.css");

#[derive(Debug, PartialEq, Clone, Copy)]
enum AppEvent {
    TestSetupSetPan(f32),
}

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for App {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, app: Entity) -> Self::Ret {
        Header::default().build(state, app, |builder| builder);
        Timeline::new().build(state, app, |builder|
            builder
                //.set_height(Pixels(300.0))
                .set_space(Pixels(2.0))
        );

        app.set_background_color(state, Color::rgb(10, 10, 10))
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {}
}

pub fn run() {
    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let sample_rate = crate::backend::hardware_io::default_sample_rate();

    let (project_interface, rt_state) =
        ProjectStateInterface::new(sample_rate);

    // This function is temporary. Eventually we should use rusty-daw-io instead.
    let _stream = crate::backend::rt_thread::run_with_default_output(rt_state);

    let window_description = WindowDescription::new().with_title("Meadowlark");
    let app = Application::new(window_description, |state, window| {
        //state.add_theme(DEFAULT_THEME);
        state.add_theme(THEME);

        //let text_to_speech = TextToSpeach::new().build(state, window, |builder| builder);

        //App data lives at the top of the tree
        let app_data = AppData::new(project_interface).build(state, window);

        App::new().build(state, app_data, |builder| builder);
    });

    app.run();
}
