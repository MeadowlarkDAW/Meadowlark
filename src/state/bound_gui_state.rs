use tuix::Lens;

#[derive(Default, Debug, Clone, Lens)]
pub struct BoundGuiState {
    pub backend_loaded: bool,
    pub is_playing: bool,
    pub bpm: f64,
}
