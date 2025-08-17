use std::path::PathBuf;

use crate::Driver;

pub fn load_driver_from(path: &str) -> (Driver, PathBuf) {
    let path = std::path::PathBuf::from(path);
    let name = PathBuf::from(path.file_stem().expect("Failed to get file stem"));
    let driver = Driver::new(path, name.display().to_string(), None).expect("Failed to create driver");
    (driver, name)
}