
use tuix::widgets::*;

use super::control_bars::*;

#[derive(Default)]
pub struct Header {

}

impl Widget for Header {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        
        TempoControlBar::default().build(state, entity, |builder| builder);
        Element::new().build(state, entity, |builder| builder
            .set_focusable(false)
            .class("spacer")
        );
        TransportControlBar::default().build(state, entity, |builder| builder);


        entity.class(state, "header").set_focusable(state, false)
    }
}