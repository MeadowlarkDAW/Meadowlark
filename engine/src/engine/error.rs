use std::error::Error;

use meadowlark_plugin_api::PluginFormat;

use crate::graph::error::GraphCompilerError;

#[derive(Debug)]
#[non_exhaustive]
pub enum EngineCrashError {
    CompilerError(GraphCompilerError),
}

impl Error for EngineCrashError {}

impl std::fmt::Display for EngineCrashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineCrashError::CompilerError(e) => {
                write!(f, "Engine crashed: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum NewPluginInstanceError {
    FactoryFailedToCreateNewInstance(String, String),
    PluginFailedToInit(String, String),
    NotFound(String),
    FormatNotFound(String, PluginFormat),
}

impl Error for NewPluginInstanceError {}

impl std::fmt::Display for NewPluginInstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewPluginInstanceError::FactoryFailedToCreateNewInstance(n, e) => {
                write!(f, "Failed to create instance of plugin {}: plugin factory failed to create new instance: {}", n, e)
            }
            NewPluginInstanceError::PluginFailedToInit(n, e) => {
                write!(f, "Failed to create instance of plugin {}: plugin instance failed to initialize: {}", n, e)
            }
            NewPluginInstanceError::NotFound(n) => {
                write!(
                    f,
                    "Failed to create instance of plugin {}: not in list of scanned plugins",
                    n
                )
            }
            NewPluginInstanceError::FormatNotFound(n, p) => {
                write!(
                    f,
                    "Failed to create instance of plugin {}: the format {:?} not found for this plugin",
                    n,
                    p
                )
            }
        }
    }
}
