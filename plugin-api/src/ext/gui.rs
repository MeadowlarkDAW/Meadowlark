pub use clack_extensions::gui::{AspectRatioStrategy, GuiResizeHints, GuiSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedGuiInfo {
    /// The initial size of this GUI.
    pub size: GuiSize,

    /// Whether or not this window can be resized.
    pub resizable: bool,

    /// Information provided by the plugin to improve window resizing
    /// when initiated by the host or window manager.
    pub resize_hints: Option<GuiResizeHints>,
}
