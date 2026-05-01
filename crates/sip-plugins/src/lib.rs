pub mod extension_points;
pub mod manifest;
pub mod registry;
pub mod validation;

pub use extension_points::{known_extension_points, ExtensionPointDef};
pub use manifest::*;
pub use registry::PluginRegistry;
pub use validation::{
    validate_manifest, validate_plugin_set, ValidationResult, KNOWN_EXTENSION_POINTS,
};
