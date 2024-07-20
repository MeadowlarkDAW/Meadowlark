use super::App;

/// Handle an action
///
/// This will only return an error if a fatal error has occured that
/// requires the application to close.
pub fn handle_action(
    action: crate::Action,
    app: &mut App,
    cx: &mut yarrow::AppContext<crate::Action>,
) -> anyhow::Result<()> {
    // TODO
    Ok(())
}
