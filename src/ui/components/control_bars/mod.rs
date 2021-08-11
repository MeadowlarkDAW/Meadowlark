use tuix::*;

pub mod tempo;
pub use tempo::*;

pub mod transport;
pub use transport::*;


pub struct ControlBar {
    name: String,
}

impl ControlBar {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Widget for ControlBar {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        
        Label::new(&self.name).build(state, entity, |builder| builder);

        let controls = Row::new().build(state, entity, |builder| builder.class("controls"));

        entity.class(state, "control_bar");

        controls
    }
}
