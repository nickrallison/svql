use std::path::PathBuf;

use crate::{cache::Cache, driver::Driver};

pub fn load_driver_from(path: &str) -> Result<Driver, Box<dyn std::error::Error>> {
    let mut cache: Cache = Cache::new();
    load_driver_cached(path, &mut cache)
}

pub fn load_driver_cached(
    path: &str,
    cache: &mut Cache,
) -> Result<Driver, Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(path);
    let name = PathBuf::from(path.file_stem().ok_or("Failed to get file stem")?)
        .to_str()
        .unwrap()
        .to_string();
    let driver = Driver::new(path, name.clone(), Some(cache))?;
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_driver() {
        let _driver = load_driver_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }
}
