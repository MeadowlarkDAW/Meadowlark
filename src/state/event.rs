use super::ProjectSaveState;

#[derive(Debug, Clone)]
pub enum StateSystemEvent {
    Transport(TransportEvent),
    Tempo(TempoEvent),
    Project(ProjectEvent),
}

// TODO: Remove this once tuix removes the `PartialEq` requirement
// on messages.
impl PartialEq for StateSystemEvent {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub enum ProjectEvent {
    LoadProject(Box<ProjectSaveState>),
}

#[derive(Debug, Clone)]
pub enum TempoEvent {
    SetBPM(f64),
}

#[derive(Debug, Clone)]
pub enum TransportEvent {
    Play,
    Stop,
    Pause,
}

impl ProjectEvent {
    pub fn to_state_event(self) -> StateSystemEvent {
        self.into()
    }
}
impl From<ProjectEvent> for StateSystemEvent {
    fn from(e: ProjectEvent) -> Self {
        Self::Project(e)
    }
}

impl TempoEvent {
    pub fn to_state_event(self) -> StateSystemEvent {
        self.into()
    }
}
impl From<TempoEvent> for StateSystemEvent {
    fn from(e: TempoEvent) -> Self {
        Self::Tempo(e)
    }
}

impl TransportEvent {
    pub fn to_state_event(self) -> StateSystemEvent {
        self.into()
    }
}
impl From<TransportEvent> for StateSystemEvent {
    fn from(e: TransportEvent) -> Self {
        Self::Transport(e)
    }
}
