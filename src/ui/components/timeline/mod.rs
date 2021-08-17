


use tuix::*;


/// A general purpose timeline widget
pub struct Timeline {

}

impl Timeline {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Widget for Timeline {
    type Ret = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        // Vertical scroll container for control and tracks
        let scroll = ScrollContainer::new().build(state, entity, |builder| builder);

        let row = Row::new().build(state, scroll, |builder| 
            builder
                .set_height(Auto)
                .set_col_between(Pixels(2.0))
        );

        // 
        let controls = Element::new().build(state, row, |builder| 
            builder
                .set_background_color(Color::rgb(20,20,20))
                //.set_text("Controls")
                .set_height(Auto)
                .set_row_between(Pixels(2.0))
        );

        for _ in 0..10 {
            Element::new().build(state, controls, |builder| 
                builder
                    .set_height(Pixels(50.0))
                    .set_background_color(Color::rgb(100, 100, 100))
            );
        }

        let tracks = Element::new().build(state, row, |builder| 
            builder
                .set_background_color(Color::rgb(20,20,20))   
                //.set_text("Tracks") 
                .set_row_between(Pixels(2.0))
                .set_height(Auto)
        );

        for _ in 0..10 {
            Element::new().build(state, tracks, |builder| 
                builder
                    .set_height(Pixels(50.0))
                    .set_background_color(Color::rgb(100, 100, 100))
            );
        }

        entity
    }
}