use std::{path::PathBuf, sync::{Arc, Mutex}};

use crate::{cache::{Cache}, Driver};

pub fn load_driver_from(path: &str) -> Result<(Driver, String), Box<dyn std::error::Error>> {
    let mut cache: Cache = Cache::new();
    load_driver_cached(path, &mut cache)
}

pub fn load_driver_cached(path: &str, cache: &mut Cache) -> Result<(Driver, String), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(path);
    let name = PathBuf::from(path.file_stem().ok_or("Failed to get file stem")?).to_str().unwrap().to_string();
    let driver = Driver::new(path, name.clone(), Some(cache))?;
    Ok((driver, name))
}
