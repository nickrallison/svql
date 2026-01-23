//! Unique identifiers for hardware designs.

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A unique identifier for a loaded design.
///
/// Identity is determined by the canonical file path and the specific module
/// name within that file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriverKey {
    /// The filesystem path to the design source.
    pub path: PathBuf,
    /// The name of the top-level module.
    pub module_name: Arc<str>,
}

impl DriverKey {
    /// Creates a new key from a path and module name.
    pub fn new<P, S>(path: P, module_name: S) -> Self
    where
        P: AsRef<Path>,
        S: Into<Arc<str>>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            module_name: module_name.into(),
        }
    }

    /// Returns a reference to the design path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the module name.
    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
