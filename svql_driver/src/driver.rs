//! Orchestration and management of hardware designs.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::debug;

use svql_common::*;
use svql_common::{DesignPath, YosysModule};
use thiserror::Error;

use crate::DriverKey;
use crate::design_container::DesignContainer;

/// Errors encountered during design ingestion or driver management.
#[derive(Debug, Error)]
pub enum DriverError {
    /// The Yosys binary could not be located.
    #[error("Failed to find yosys binary: {0}")]
    YosysNotFound(String),
    /// An I/O error occurred during file access.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// An error occurred while parsing or importing the design.
    #[error("Design loading error: {0}")]
    DesignLoading(String),
}

/// The central manager for loading, caching, and indexing designs.
#[derive(Debug, Clone)]
pub struct Driver {
    /// Thread-safe registry of loaded designs.
    registry: Arc<RwLock<HashMap<DriverKey, Arc<DesignContainer>>>>,
    /// Path to the Yosys executable.
    yosys_path: PathBuf,
    /// The root directory for resolving relative design paths.
    root_path: PathBuf,
}

impl Driver {
    /// Creates a new driver instance with a specified root path.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, DriverError> {
        let yosys = which::which("yosys").map_err(|e| DriverError::YosysNotFound(e.to_string()))?;

        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys_path: yosys,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    /// Updates the path to the Yosys binary.
    pub fn set_yosys_path<P: AsRef<Path>>(&mut self, yosys: P) {
        self.yosys_path = yosys.as_ref().to_path_buf();
    }

    /// Creates a driver rooted at the current Cargo workspace.
    pub fn new_workspace() -> Result<Self, DriverError> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::new(workspace)
    }

    /// Creates a driver rooted at the workspace with a specific Yosys path.
    pub fn new_workspace_yosys<Y: AsRef<Path>>(yosys: Y) -> Result<Self, DriverError> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::with_yosys(workspace, yosys)
    }

    /// Creates a driver with explicit root and Yosys paths.
    pub fn with_yosys<P: AsRef<Path>, Y: AsRef<Path>>(
        root: P,
        yosys: Y,
    ) -> Result<Self, DriverError> {
        let yosys_path = yosys.as_ref().to_path_buf();
        if !yosys_path.exists() {
            return Err(DriverError::YosysNotFound(format!(
                "Yosys binary not found at: {}",
                yosys_path.display()
            )));
        }

        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys_path,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    /// Internal helper to resolve a design path to an absolute path.
    fn resolve_path(&self, design_path: &Path) -> PathBuf {
        if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            self.root_path.join(design_path)
        }
    }

    /// Internal helper to check if a key exists in the registry.
    fn check_registry(&self, key: &DriverKey) -> Option<Arc<DesignContainer>> {
        self.registry.read().unwrap().get(key).cloned()
    }

    pub fn preload_design(
        &self,
        key: &DriverKey,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<(), DriverError> {
        Self::get_design(self, key, module_config)?;
        Ok(())
    }

    pub fn get_design(
        &self,
        key: &DriverKey,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<Arc<DesignContainer>, DriverError> {
        if let Some(design) = self.check_registry(key) {
            debug!("Design already loaded: {:?}", key);
            return Ok(design);
        }

        let absolute_path = self.resolve_path(key.path());
        let yosys_module = YosysModule::new(&absolute_path, key.module_name())
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = if module_config.load_raw {
            if !matches!(yosys_module.design_path(), DesignPath::Json(_)) {
                return Err(DriverError::DesignLoading(
                    "Raw loading is only supported for JSON netlists.".to_string(),
                ));
            }
            yosys_module
                .import_design_raw()
                .map_err(|e| DriverError::DesignLoading(e.to_string()))?
        } else {
            yosys_module
                .import_design(module_config)
                .map_err(|e| DriverError::DesignLoading(e.to_string()))?
        };

        let design_container = Arc::new(DesignContainer::build(design));

        self.registry
            .write()
            .unwrap()
            .insert(key.clone(), design_container.clone());

        Ok(design_container)
    }

    /// Returns a copy of all currently loaded designs.
    #[must_use]
    pub fn get_all_designs(&self) -> HashMap<DriverKey, Arc<DesignContainer>> {
        self.registry.read().unwrap().clone()
    }
}
