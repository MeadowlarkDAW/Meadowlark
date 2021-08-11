use tuix::*;

pub struct LevelsMeter {
    front: Entity,

    level: f32,
}

impl LevelsMeter {
    pub fn new() -> Self {
        Self {
            front: Entity::null(),

            level: 0.0,
        }
    }
}

impl Widget for LevelsMeter {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let front = Element::new().build(state, entity, |builder| {
            builder
                .set_height(Percentage(0.0))
                .set_background_color(Color::green())
        });

        entity
    }
}
