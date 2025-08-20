use std::path::Path;

use crate::{driver::DesignKey, driver::Driver};

/// Create a new shared driver (design registry).
pub fn new_shared_driver() -> Result<Driver, Box<dyn std::error::Error>> {
    Driver::new()
}

/// Load a design into the given shared driver; module name defaults to file stem.
/// Returns the DesignKey.
pub fn ensure_loaded<P: AsRef<Path>>(
    driver: &Driver,
    path: P,
) -> Result<DesignKey, Box<dyn std::error::Error>> {
    driver.ensure_loaded(path)
}

/// Load a design into the given shared driver with explicit top module name.
/// Returns the DesignKey.
pub fn ensure_loaded_with_top<P: Into<std::path::PathBuf>>(
    driver: &Driver,
    path: P,
    top: impl Into<String>,
) -> Result<DesignKey, Box<dyn std::error::Error>> {
    driver.ensure_loaded_with_top(path.into(), top.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_design_key() {
        let driver = new_shared_driver().unwrap();
        let key = ensure_loaded(&driver, "examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        let d = driver.get(&key).unwrap();
        assert!(d.iter_cells().count() > 0);
    }
}
