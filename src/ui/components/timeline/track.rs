
use tuix::*;

//use crate::backend::timeline::TimelineTrackSaveState;

// Track (TODO)
pub struct Track {
    name: String,
}

impl Track {
    pub fn new(name: String) -> Self {
        Self {
            name: name.clone(),
        }
    }
}

impl Widget for Track {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_background_color(state, Color::rgb(150, 100, 190))
            .set_height(state, Pixels(80.0))
            .set_width(state, Pixels(1000.0))
            .set_text(state, &self.name)
    }
}


// Track Controls (TODO)

pub struct TrackControls {

}

impl TrackControls {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Widget for TrackControls {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_background_color(state, Color::rgb(100, 150, 100))
            .set_height(state, Pixels(80.0))
            .set_width(state, Pixels(1000.0))
    }
}