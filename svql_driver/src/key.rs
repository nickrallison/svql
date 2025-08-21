use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriverKey {
    pub path: PathBuf,
    pub module_name: Arc<str>,
}

impl DriverKey {
    pub fn new<P: AsRef<Path>>(path: P, module_name: String) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            module_name: Arc::from(module_name),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
