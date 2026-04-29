use std::collections::HashMap;

pub type ExtensionPoint = String;
pub type ExtensionRegistry = HashMap<ExtensionPoint, Vec<String>>;

pub fn default_extension_points() -> ExtensionRegistry {
    HashMap::new()
}
