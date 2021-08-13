


use tuix::*;


/// A general purpose timeline widget
pub struct Timeline {

}

impl Timeline {
    pub fn new() -> Self {

    }
}

impl Widget for Timeline {
    type Ret = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        // Vertical scroll container for control and tracks
        ScrollContainerV::new().build(state, entity, |builder| builder);

        Element::new().build(state, entity, |builder| builder.set_background_color(Color::red()));

        Element::new().build(state, entity, |builder| builder.set_background_color(Color::green()));

        entity
    }
}