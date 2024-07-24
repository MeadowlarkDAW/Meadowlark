use yarrow::{TooltipInfo, WindowID};

#[derive(Clone)]
pub enum AppAction {
    ShowTooltip {
        window_id: WindowID,
        info: TooltipInfo,
    },
    HideTooltip {
        window_id: WindowID,
    },
}
