use std::{path::PathBuf, sync::{Arc, Mutex}};

use crate::{cache::Cache, Driver};

pub fn load_driver_from(path: &str) -> (Driver, String) {
    let path = std::path::PathBuf::from(path);
    let name = PathBuf::from(path.file_stem().expect("Failed to get file stem")).to_str().unwrap().to_string();
    let driver = Driver::new(path, name.clone(), None).expect("Failed to create driver");
    (driver, name)
}

pub fn load_driver_cached(path: &str, cache: Arc<Mutex<Cache>>) -> (Driver, String) {
    let path = std::path::PathBuf::from(path);
    let name = PathBuf::from(path.file_stem().expect("Failed to get file stem")).to_str().unwrap().to_string();
    let mut cache_guard = cache.lock().expect("Failed to lock cache");
    let driver = Driver::new(path, name.clone(), Some(&mut *cache_guard)).expect("Failed to create driver");
    (driver, name)
}