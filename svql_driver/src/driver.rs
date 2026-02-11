//! Design driver for loading, caching, and managing hardware designs.
//!
//! The `Driver` is responsible for:
//! - Loading hardware designs from disk (Verilog, RTLIL, JSON)
//! - Managing a registry to prevent redundant design reloads
//! - Creating graph indices for subgraph matching
//! - Providing access to loaded designs through design containers

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

use svql_common::*;
use svql_common::{DesignPath, YosysModule};
use thiserror::Error;

use crate::DriverKey;
use crate::design_container::DesignContainer;

/// Errors that can occur during design ingestion and driver management.
#[derive(Debug, Error)]
pub enum DriverError {
    /// The Yosys binary could not be located on the system.
    #[error("Failed to find yosys binary: {0}")]
    YosysNotFound(String),
    /// An I/O error occurred during file access.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// An error occurred while parsing or importing the design.
    #[error("Design loading error: {0}")]
    DesignLoading(String),
}

/// Central manager for loading, caching, and indexing hardware designs.
///
/// The driver maintains a thread-safe registry to cache loaded designs
/// and provides methods to load designs from files with optional Yosys processing.
#[derive(Debug, Clone)]
pub struct Driver {
    /// Thread-safe registry mapping design keys to loaded design containers
    registry: Arc<RwLock<HashMap<DriverKey, Arc<DesignContainer>>>>,
    /// Path to the Yosys executable
    yosys_path: PathBuf,
    /// Root directory for resolving relative design paths
    root_path: PathBuf,
}

impl Driver {
    /// Creates a new driver instance rooted at the specified directory.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory for resolving relative design paths
    ///
    /// # Errors
    ///
    /// Returns `DriverError::YosysNotFound` if the yosys executable cannot be located in PATH.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, DriverError> {
        let yosys = which::which("yosys").map_err(|e| DriverError::YosysNotFound(e.to_string()))?;

        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys_path: yosys,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    /// Updates the path to the Yosys executable.
    pub fn set_yosys_path<P: AsRef<Path>>(&mut self, yosys: P) {
        self.yosys_path = yosys.as_ref().to_path_buf();
    }

    /// Creates a driver rooted at the current Cargo workspace.
    ///
    /// # Errors
    ///
    /// Returns `DriverError::YosysNotFound` if yosys is not in PATH or `DriverError::Io`
    /// if the workspace directory cannot be canonicalized.
    pub fn new_workspace() -> Result<Self, DriverError> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::new(workspace)
    }

    /// Creates a driver rooted at the workspace with an explicit Yosys path.
    ///
    /// # Arguments
    ///
    /// * `yosys` - Path to the Yosys executable
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be canonicalized or if the yosys path is invalid.
    pub fn new_workspace_yosys<Y: AsRef<Path>>(yosys: Y) -> Result<Self, DriverError> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::with_yosys(workspace, yosys)
    }

    /// Creates a driver with explicit root and Yosys paths.
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory for resolving relative paths
    /// * `yosys` - Path to the Yosys executable
    ///
    /// # Errors
    ///
    /// Returns `DriverError::YosysNotFound` if the yosys binary path does not exist.
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

    /// Converts a relative or absolute path to an absolute path.
    fn resolve_path(&self, design_path: &Path) -> PathBuf {
        if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            self.root_path.join(design_path)
        }
    }

    /// Checks the registry for an already-loaded design.
    fn check_registry(&self, key: &DriverKey) -> Option<Arc<DesignContainer>> {
        self.registry.read().unwrap().get(key).cloned()
    }

    /// Preloads a design into the registry without waiting for query execution.
    ///
    /// # Arguments
    ///
    /// * `key` - Design identifier
    /// * `module_config` - Configuration for design processing
    ///
    /// # Errors
    ///
    /// Returns a `DriverError` if the design cannot be loaded.
    pub fn preload_design(
        &self,
        key: &DriverKey,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<(), DriverError> {
        Self::get_design(self, key, module_config)?;
        Ok(())
    }

    /// Loads or retrieves a cached design.
    ///
    /// If the design is not already in the registry, it is loaded from disk,
    /// processed via Yosys (unless raw loading is enabled), and cached.
    ///
    /// # Arguments
    ///
    /// * `key` - Design identifier containing path and module name
    /// * `module_config` - Configuration for design processing and optimization
    ///
    /// # Errors
    ///
    /// Returns a `DriverError` if the design file is invalid, cannot be read,
    /// or if Yosys processing fails.
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

        info!(
            "Design loaded successfully: {:?}, module: {}",
            key.path(),
            key.module_name()
        );
        let design_container = Arc::new(DesignContainer::build(design));

        self.registry
            .write()
            .unwrap()
            .insert(key.clone(), design_container.clone());

        Ok(design_container)
    }

    /// Returns a snapshot of all currently loaded designs.
    #[must_use]
    pub fn get_all_designs(&self) -> HashMap<DriverKey, Arc<DesignContainer>> {
        self.registry.read().unwrap().clone()
    }

    /// Retrieves source location logic using the stable PhysicalCellId.
    pub fn get_cell_source(
        &self,
        key: &DriverKey,
        physical: PhysicalCellId,
    ) -> Option<SourceLocation> {
        let container = {
            let registry = self.registry.read().ok()?;
            registry.get(key)?.clone()
        };

        // Translate physical back to local node to reach the cell wrapper
        let node = container.index().resolve_node(physical)?;
        container.index().get_cell_by_index(node).get_source()
    }
}
