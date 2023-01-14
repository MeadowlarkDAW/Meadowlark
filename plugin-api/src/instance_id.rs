use basedrop::Shared;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PluginFormat {
    Internal,
    Clap,
}

impl std::fmt::Display for PluginFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginFormat::Internal => {
                write!(f, "internal")
            }
            PluginFormat::Clap => {
                write!(f, "CLAP")
            }
        }
    }
}

/// Used for debugging and verifying purposes.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginInstanceType {
    Internal,
    Clap,
    Unloaded,
    GraphInput,
    GraphOutput,
}

impl std::fmt::Debug for PluginInstanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PluginInstanceType::Internal => "INT",
                PluginInstanceType::Clap => "CLAP",
                PluginInstanceType::Unloaded => "UNLOADED",
                PluginInstanceType::GraphInput => "GRAPH_IN",
                PluginInstanceType::GraphOutput => "GRAPH_OUT",
            }
        )
    }
}

impl From<PluginFormat> for PluginInstanceType {
    fn from(f: PluginFormat) -> Self {
        match f {
            PluginFormat::Internal => PluginInstanceType::Internal,
            PluginFormat::Clap => PluginInstanceType::Clap,
        }
    }
}

/// A unique ID for a plugin instance.
pub struct PluginInstanceID {
    node_id: u32,
    // To make sure that no two plugin instances ever have the same ID.
    unique_id: u64,
    format: PluginInstanceType,
    rdn: Shared<String>,
}

impl PluginInstanceID {
    pub fn _new(
        node_id: u32,
        unique_id: u64,
        format: PluginInstanceType,
        rdn: Shared<String>,
    ) -> Self {
        Self { node_id, unique_id, format, rdn }
    }

    /// The reverse domain name of this plugin.
    pub fn rdn(&self) -> &Shared<String> {
        &self.rdn
    }

    pub fn unique_id(&self) -> u64 {
        self.unique_id
    }

    pub fn format(&self) -> PluginInstanceType {
        self.format
    }

    pub fn _node_id(&self) -> u32 {
        self.node_id
    }
}

impl std::fmt::Debug for PluginInstanceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            PluginInstanceType::Internal => {
                write!(f, "INT({})({})", &**self.rdn, self.unique_id)
            }
            PluginInstanceType::Clap => {
                write!(f, "CLAP({})({})", &**self.rdn, self.unique_id)
            }
            PluginInstanceType::Unloaded => {
                write!(f, "UNLOADED({})", self.unique_id)
            }
            PluginInstanceType::GraphInput => {
                write!(f, "GRAPH_IN")
            }
            PluginInstanceType::GraphOutput => {
                write!(f, "GRAPH_OUT")
            }
        }
    }
}

impl Clone for PluginInstanceID {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            unique_id: self.unique_id,
            format: self.format,
            rdn: Shared::clone(&self.rdn),
        }
    }
}

impl PartialEq for PluginInstanceID {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id.eq(&other.unique_id)
    }
}

impl Eq for PluginInstanceID {}

impl Hash for PluginInstanceID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //self.node_id.hash(state);
        self.unique_id.hash(state);
    }
}
