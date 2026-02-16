#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use std::fs;
use std::path::PathBuf;
use svql_common::ModuleConfig;
use svql_driver::{Driver, DriverKey};
use tempfile::tempdir;

#[test]
fn test_driver_initialization() {
    let tmp = tempdir().unwrap();
    let driver = Driver::new(tmp.path()).expect("Failed to create driver");
    assert_eq!(driver.root_path(), fs::canonicalize(tmp.path()).unwrap());
}

#[test]
fn test_driver_resolve_path() {
    let tmp = tempdir().unwrap();
    let driver = Driver::new(tmp.path()).unwrap();

    let rel_path = PathBuf::from("designs/my_chip.v");
    let resolved = driver.resolve_path(&rel_path);

    assert!(resolved.is_absolute());
    assert!(resolved.ends_with("designs/my_chip.v"));
}

#[test]
fn test_driver_preload_invalid_path() {
    let tmp = tempdir().unwrap();
    let driver = Driver::new(tmp.path()).unwrap();
    let key = DriverKey::new("non_existent.v", "top");
    let config = ModuleConfig::default();

    let result = driver.preload_design(&key, &config);
    assert!(result.is_err());
}
