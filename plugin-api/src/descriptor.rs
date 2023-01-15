/// The description of a plugin.
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    /// The unique reverse-domain-name identifier of this plugin.
    ///
    /// eg: "org.rustydaw.spicysynth"
    pub id: String,

    /// The version of this plugin.
    ///
    /// eg: "1.4.4" or "1.1.2_beta"
    pub version: String,

    /// The displayable name of this plugin.
    ///
    /// eg: "Spicy Synth"
    pub name: String,

    /// The vendor of this plugin.
    ///
    /// eg: "RustyDAW"
    pub vendor: String,

    /// A displayable short description of this plugin.
    ///
    /// eg: "Create flaming-hot sounds!"
    pub description: String,

    /// Arbitrary list of keywords, separated by `;'.
    ///
    /// They can be matched by the host search engine and used to classify the plugin.
    ///
    /// Some pre-defined keywords:
    /// - "instrument", "audio_effect", "note_effect", "analyzer"
    /// - "mono", "stereo", "surround", "ambisonic"
    /// - "distortion", "compressor", "limiter", "transient"
    /// - "equalizer", "filter", "de-esser"
    /// - "delay", "reverb", "chorus", "flanger"
    /// - "tool", "utility", "glitch"
    ///
    /// Some examples:
    /// - "equalizer;analyzer;stereo;mono"
    /// - "compressor;analog;character;mono"
    /// - "reverb;plate;stereo"
    pub features: String,

    /// The url to the product page of this plugin.
    pub url: String,

    /// The url to the online manual for this plugin.
    pub manual_url: String,

    /// The url to the online support page for this plugin.
    pub support_url: String,
}
