//! Unique identifiers for hardware design specifications.
//!
//! A `DriverKey` uniquely identifies a design by its file path and top-level module name.
//! Keys are used to index designs in the driver's cache to avoid redundant reloading.

use std::path::{Path, PathBuf};
use contracts::*;

/// Unique identifier for a hardware design.
///
/// Identifies a design by combining the file path and the top-level module name.
/// Two designs are considered the same if they have the same path and module name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriverKey {
    /// Filesystem path to the design file (Verilog, RTLIL, or JSON)
    pub path: PathBuf,
    /// Name of the top-level module in the design
    pub module_name: String,
}

impl DriverKey {
    /// Creates a new design key.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to the design
    /// * `module_name` - Name of the top-level module
    #[requires(!module_name.as_ref().is_empty())]
    pub fn new<P, S>(path: P, module_name: S) -> Self
    where
        P: Into<PathBuf>,
        S: AsRef<str> + Into<String>,
    {
        Self {
            path: path.into(),
            module_name: module_name.into(),
        }
    }

    /// Returns a reference to the design file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the top-level module name.
    #[must_use]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
