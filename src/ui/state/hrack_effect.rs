use vizia::prelude::*;

/// An effect on the horizontal effect rack.
#[derive(Debug, Lens, Clone, Data)]
pub enum HRackEffectState {
    Internal(InternalEffectState),
    External(ExternalEffectState),
}

#[derive(Debug, Clone, PartialEq, Data)]
pub enum InternalEffectState {
    // TODO
    Todo,
}

#[derive(Debug, Lens, Clone, Data)]
pub struct ExternalEffectState {
    pub name: String,

    /// The reverse-domain-name that uniquely identifies this plugin.
    ///
    /// (i.e. "app.meadowlark.spicy-synth")
    pub rdn: String,

    /// The version of this plugin.
    ///
    /// (i.e. "1.1.3", "1.0.2beta")
    pub version: String,

    /// The URL to the product page for this plugin (if the plugin supports it).
    pub product_url: Option<String>,
    /// The URL to the manual for this plugin (if the plugin supports it).
    pub manual_url: Option<String>,
    /// The URL to the support page for this plugin (if the plugin supports it).
    pub support_url: Option<String>,

    /// If this is true, then the effect has been collapsed into a thin bar in
    /// the horizontal effects rack.
    pub collapsed: bool,

    pub status: ActivatedStatus,

    /// True if this plugin has a custom GUI, false if not.
    pub has_gui: bool,

    /// True if the plugin's GUI is currently open, false if not.
    pub gui_is_open: bool,

    /// True if the plugin is currently bypassed.
    pub bypassed: bool,

    /// The amount of delay this plugin is creating in samples.
    pub delay: u32,

    /// The name of the currently selected preset (`None` if there is none).
    pub preset_name: Option<String>,

    /// If true, show an asterik next to the preset name to show that the
    /// preset has been modified and not saved yet.
    pub preset_changed: bool,

    /// The latest-tweaked parameter. Show this at the top of the parameter list
    /// for quick access.
    pub last_tweaked_parameter: Option<ParameterState>,

    /// If the plugin supports it, these are the parameters it wishes to show
    /// to the user for quick access. Show these before "all_parameters", but
    /// after the "last_tweaked_parameter".
    pub quick_access_parameters: Vec<ParameterState>,

    /// The list of all parameters will be hidden by default since keeping track
    /// of them creates some overhead in the backend.
    pub all_parameters_shown: bool,

    /// A list of all the parameters.
    ///
    /// This will be empty when "all_parameters_shown" is false.
    pub all_parameters: Vec<ParameterState>,
}

#[derive(Debug, Clone, Data)]
pub enum ActivatedStatus {
    /// The plugin is successfully activated an running.
    Activated,
    /// The plugin is currently deactivated due to user request. Grey out all
    /// controls on this plugin, and add an "activate" button.
    ///
    /// This is different from the plugin just being bypassed. A deactivated plugin
    /// is actually unloaded in the realtime thread.
    Deactivated,
    /// The plugin failed to activate to due an error. Grey out all controls on
    /// this plugin, and add an "retry" button.
    DeactivatedDueToError { error_msg: String },
}

#[derive(Debug, Lens, Clone, Data)]
pub enum AllParametersState {
    /// The parameters are currently hidden. This should be used by default since
    /// having them enabled creates some overhead in the backend.
    Hidden,
    /// The parameters are all currently
    Shown(Vec<ParameterState>),
}

#[derive(Debug, Lens, Clone, Data)]
pub struct ParameterState {
    pub name: String,

    pub id: u32,

    /// The string of text that displays the current value (i.e. "-12.0dB"). This
    /// is formatted by the plugin itself.
    pub display_value: String,

    /// The string of text that displays the minimum value (i.e. "-12.0dB"). This
    /// is formatted by the plugin itself.
    pub min_display_value: String,

    /// The string of text that displays the minimum value (i.e. "-12.0dB"). This
    /// is formatted by the plugin itself.
    pub max_display_value: String,

    /// The current normalized value of this parameter in the range [0.0, 1.0].
    pub normalized_value: f64,
    // TODO: Automation range
}
