


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
                .set_width(Pixels(300.0))
                .set_row_between(Pixels(2.0))
        );

        for _ in 0..10 {
            Element::new().build(state, controls, |builder| 
                builder
                    .set_height(Pixels(50.0))
                    .set_background_color(Color::rgb(100, 100, 100))
            );
        }

        let tracks = ScrollContainerH::new().disable_scroll_wheel().build(state, row, |builder| 
            builder
                //.set_background_color(Color::rgb(20,200,20))   
                //.set_text("Tracks")
        );

        tracks.set_row_between(state, Pixels(2.0));

        // tracks
        //     .set_width(state, Pixels(1000.0))
        //     .set_height(state, Pixels(100.0))
        //     .set_background_color(state, Color::red())
        //     .set_text(state, "Test");

        for _ in 0..10 {
            Element::new().build(state, tracks, |builder| 
                builder
                    .set_height(Pixels(50.0))
                    .set_width(Pixels(1000.0))
                    .set_background_color(Color::rgb(100, 100, 100))
                    .set_clip_widget(tracks)
            );
        }

        //ScrollBar::new().build(state, entity, |builder| builder);

        entity
    }
}