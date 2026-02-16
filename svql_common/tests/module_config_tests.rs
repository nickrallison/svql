#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use svql_common::ModuleConfig;

mod common;

#[test]
fn test_module_config_default() {
    let config = ModuleConfig::default();
    assert!(!config.flatten);
    assert!(!config.opt_clean);
    assert!(!config.opt);
    assert!(config.params.is_empty());
}

#[test]
fn test_module_config_builder() {
    let config = ModuleConfig::new()
        .with_flatten(true)
        .with_opt_clean(true)
        .with_param("WIDTH", "32");

    assert!(config.flatten);
    assert!(config.opt_clean);
    assert_eq!(config.params.get("WIDTH"), Some(&"32".to_string()));
}

#[test]
fn test_module_config_hash_consistency() {
    let config1 = ModuleConfig::new().with_flatten(true);
    let config2 = ModuleConfig::new().with_flatten(true);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    config1.hash(&mut hasher1);
    config2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_module_config_hash_difference() {
    let config1 = ModuleConfig::new().with_flatten(true);
    let config2 = ModuleConfig::new().with_flatten(false);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    config1.hash(&mut hasher1);
    config2.hash(&mut hasher2);

    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_module_config_equality() {
    let config1 = ModuleConfig::new().with_flatten(true).with_param("N", "4");
    let config2 = ModuleConfig::new().with_flatten(true).with_param("N", "4");

    assert_eq!(config1, config2);
}

#[test]
fn test_module_config_empty_params() {
    let config = ModuleConfig::new();
    assert!(config.params.is_empty());
}

#[test]
fn test_module_config_duplicate_params() {
    let config = ModuleConfig::new()
        .with_param("N", "4")
        .with_param("N", "8");

    assert_eq!(config.params.get("N"), Some(&"8".to_string()));
    assert_eq!(config.params.len(), 1);
}
