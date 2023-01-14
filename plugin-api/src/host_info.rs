use clack_host::host::HostInfo as ClackHostInfo;

#[derive(Debug, Clone)]
pub struct HostInfo {
    /// The name of this host (mandatory).
    ///
    /// eg: "Meadowlark"
    pub name: String,

    /// The version of this host (mandatory).
    ///
    /// eg: "1.4.4", "1.0.2_beta"
    pub version: String,

    /// The vendor of this host.
    ///
    /// eg: "RustyDAW Org"
    pub vendor: Option<String>,

    /// The url to the product page of this host.
    ///
    /// eg: "https://meadowlark.app"
    pub url: Option<String>,

    pub clack_host_info: ClackHostInfo,
}

impl HostInfo {
    /// Create info about this host.
    ///
    /// - `name` - The name of this host (mandatory). eg: "Meadowlark"
    /// - `version` - The version of this host (mandatory). eg: "1.4.4", "1.0.2_beta"
    ///     - A quick way to do this is to set this equal to `String::new(env!("CARGO_PKG_VERSION"))`
    ///     to automatically update this when your crate version changes.
    ///
    /// - `vendor` - The vendor of this host. eg: "RustyDAW Org"
    /// - `url` - The url to the product page of this host. eg: "https://meadowlark.app"
    pub fn new(name: String, version: String, vendor: Option<String>, url: Option<String>) -> Self {
        let clack_host_info = ClackHostInfo::new(
            &name,
            vendor.as_deref().unwrap_or(""),
            url.as_deref().unwrap_or(""),
            &version,
        )
        .unwrap();

        Self { name, version, vendor, url, clack_host_info }
    }
}
