use vizia::prelude::*;

// NOTE - This might be the same as Clip

#[derive(Debug, Clone, Lens, Data, Serialize, Deserialize)]
pub struct PatternState {
    pub name: String,
    pub channel: usize,
}

impl Model for PatternState {}
