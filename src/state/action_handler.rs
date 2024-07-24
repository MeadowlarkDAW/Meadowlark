use yarrow::MAIN_WINDOW;

use super::{App, AppAction};

/// Handle an action
///
/// This will only return an error if a fatal error has occured that
/// requires the application to close.
pub fn handle_action(
    action: crate::AppAction,
    app: &mut App,
    cx: &mut yarrow::AppContext<crate::AppAction>,
) -> anyhow::Result<()> {
    let App { main_window, .. } = app;

    let main_window = main_window.as_mut().unwrap();

    match action {
        AppAction::ShowTooltip { window_id, info } => match window_id {
            MAIN_WINDOW => main_window.show_tooltip(info, cx),
            _ => {}
        },
        AppAction::HideTooltip { window_id } => match window_id {
            MAIN_WINDOW => main_window.hide_tooltip(),
            _ => {}
        },
    }

    Ok(())
}
