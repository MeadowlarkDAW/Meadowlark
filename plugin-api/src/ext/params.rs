use bitflags::bitflags;
use clack_host::utils::Cookie;
use std::hash::Hash;

pub use clack_extensions::params::{ParamClearFlags, ParamRescanFlags};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamID(pub u32);

impl ParamID {
    pub const fn new(stable_id: u32) -> Self {
        Self(stable_id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

bitflags! {
    pub struct ParamInfoFlags: u32 {
        /// Is this param stepped? (integer values only)
        ///
        /// If so the double value is converted to integer using a cast (equivalent to trunc).
        const IS_STEPPED = 1 << 0;

        /// Useful for for periodic parameters like a phase.
        const IS_PERIODIC = 1 << 1;

        /// The parameter should not be shown to the user, because it is currently not used.
        ///
        /// It is not necessary to process automation for this parameter.
        const IS_HIDDEN = 1 << 2;

        /// The parameter can't be changed by the host.
        const IS_READONLY = 1 << 3;

        /// This parameter is used to merge the plugin and host bypass button.
        ///
        /// It implies that the parameter is stepped.
        ///
        /// - min: 0 -> bypass off
        /// - max: 1 -> bypass on
        const IS_BYPASS = 1 << 4;

        /// When set:
        /// - automation can be recorded
        /// - automation can be played back
        ///
        /// The host can send live user changes for this parameter regardless of this flag.
        ///
        /// If this parameters affect the internal processing structure of the plugin, ie: max delay, fft
        /// size, ... and the plugins needs to re-allocate its working buffers, then it should call
        /// host->request_restart(), and perform the change once the plugin is re-activated.
        const IS_AUTOMATABLE = 1 << 5;

        /// Does this param support per note automations?
        const IS_AUTOMATABLE_PER_NOTE_ID = 1 << 6;

        /// Does this param support per note automations?
        const IS_AUTOMATABLE_PER_KEY = 1 << 7;

        /// Does this param support per channel automations?
        const IS_AUTOMATABLE_PER_CHANNEL = 1 << 8;

        /// Does this param support per port automations?
        const IS_AUTOMATABLE_PER_PORT = 1 << 9;

        /// Does the parameter support the modulation signal?
        const IS_MODULATABLE = 1 << 10;

        /// Does this param support per note automations?
        const IS_MODULATABLE_PER_NOTE_ID = 1 << 11;

        /// Does this param support per note automations?
        const IS_MODULATABLE_PER_KEY = 1 << 12;

        /// Does this param support per channel automations?
        const IS_MODULATABLE_PER_CHANNEL = 1 << 13;

        /// Does this param support per channel automations?
        const IS_MODULATABLE_PER_PORT = 1 << 14;

        /// Any change to this parameter will affect the plugin output and requires to be done via
        /// process() if the plugin is active.
        ///
        /// A simple example would be a DC Offset, changing it will change the output signal and must be
        /// processed.
        const REQUIRES_PROCESS = 1 << 15;
    }
}

impl ParamInfoFlags {
    /// `Self::IS_AUTOMATABLE | Self::IS_MODULATABLE`
    pub fn default_float() -> Self {
        Self::IS_AUTOMATABLE | Self::IS_MODULATABLE
    }

    /// `Self::IS_STEPPED | Self::IS_AUTOMATABLE | Self::IS_MODULATABLE`
    pub fn default_enum() -> Self {
        Self::IS_STEPPED | Self::IS_AUTOMATABLE | Self::IS_MODULATABLE
    }
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    /// Stable parameter identifier, it must never change.
    pub stable_id: ParamID,

    pub flags: ParamInfoFlags,

    /// The name of this parameter displayed to the user.
    pub display_name: String,

    /// The module containing the param.
    ///
    /// eg: `"oscillators/wt1"`
    ///
    /// `/` will be used as a separator to show a tree like structure.
    pub module: String,

    /// Minimum plain value.
    pub min_value: f64,
    /// Maximum plain value.
    pub max_value: f64,
    /// Default plain value.
    pub default_value: f64,

    /// Reserved for CLAP plugins.
    #[allow(unused)]
    pub _cookie: Cookie,
}

impl ParamInfo {
    /// Create info for a parameter.
    ///
    /// - `stable_id` - Stable parameter identifier, it must never change.
    /// - `flags` - Additional flags.
    /// - `display_name` - The name of this parameter displayed to the user.
    /// - `module` - The module containing the param.
    ///     - eg: `"oscillators/wt1"`
    ///     - `/` will be used as a separator to show a tree like structure.
    /// - `min_value`: Minimum plain value.
    /// - `max_value`: Maximum plain value.
    /// - `default_value`: Default plain value.
    pub fn new(
        stable_id: ParamID,
        flags: ParamInfoFlags,
        display_name: String,
        module: String,
        min_value: f64,
        max_value: f64,
        default_value: f64,
    ) -> Self {
        Self {
            stable_id,
            flags,
            display_name,
            module,
            min_value,
            max_value,
            default_value,
            _cookie: Cookie::empty(),
        }
    }
}
