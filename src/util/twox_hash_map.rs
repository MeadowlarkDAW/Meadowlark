use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use twox_hash::XxHash64;

/// A hashmap using the XXHash algorithm.
///
/// For more information, see https://crates.io/crates/twox-hash
pub type TwoXHashMap<K, V> = HashMap<K, V, BuildHasherDefault<XxHash64>>;