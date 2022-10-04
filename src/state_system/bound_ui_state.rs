use vizia::prelude::*;

#[derive(Debug, Lens, Clone)]
pub struct BoundUiState {}

impl BoundUiState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Model for BoundUiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}
