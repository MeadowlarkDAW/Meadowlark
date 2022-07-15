use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    // ----- General -----
    PollEngine,

    // Project
    SaveProject,
    LoadProject,

    // ----- Channel Rack -----
    SelectChannel(usize),

    // ----- Timeline -----

    // Insertion
    InsertLane,
    DuplicateSelectedLanes,

    // Selection
    SelectLane(usize),
    SelectLaneAbove,
    SelectLaneBelow,
    SelectAllLanes,
    MoveSelectedLanesUp,
    MoveSelectedLanesDown,

    // Deletion
    DeleteSelectedLanes,
    ToggleLaneActivation,

    // Zoom
    ZoomInVertically,
    ZoomOutVertically,

    // Height
    IncreaseSelectedLaneHeight,
    DecreaseSelectedLaneHeight,

    // Activation
    ActivateSelectedLanes,
    DeactivateSelectedLanes,
    ToggleSelectedLaneActivation,

    // ----- Browser -----
    SetBrowserWidth(f32),
    BrowserFileClicked(PathBuf),
    BrowserFileStop(),
}
