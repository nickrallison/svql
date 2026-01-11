//! Orchestration and management of hardware designs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use svql_common::YosysModule;
use thiserror::Error;

use crate::design_container::DesignContainer;
use crate::{Context, DriverKey};

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

    /// Loads a design into the registry.
    #[contracts::debug_requires(!module_name.is_empty())]
    pub fn load_design<P: AsRef<Path>>(
        &self,
        design_path: P,
        module_name: String,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<DriverKey, DriverError> {
        let absolute_path = self.resolve_path(design_path.as_ref());
        let key = DriverKey::new(&absolute_path, module_name.clone());

        if let Some(_) = self.check_registry(&key) {
            return Ok(key);
        }

        let yosys_module = YosysModule::new(&absolute_path.display().to_string(), &module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = yosys_module
            .import_design_yosys(module_config, &self.yosys_path)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        self.registry
            .write()
            .unwrap()
            .insert(key.clone(), Arc::new(DesignContainer::build(design)));
        Ok(key)
    }

    /// Retrieves a design from the registry or loads it if missing.
    #[contracts::debug_requires(!module_name.is_empty())]
    pub fn get_or_load_design(
        &self,
        design_path: &str,
        module_name: &str,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<(DriverKey, Arc<DesignContainer>), DriverError> {
        let absolute_path = self.resolve_path(Path::new(design_path));
        let key = DriverKey::new(&absolute_path, module_name.to_string());

        if let Some(design) = self.check_registry(&key) {
            return Ok((key, design));
        }

        let yosys_module = YosysModule::new(&absolute_path.display().to_string(), module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = yosys_module
            .import_design_yosys(module_config, &self.yosys_path)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        let design_arc = Arc::new(DesignContainer::build(design));
        self.registry
            .write()
            .unwrap()
            .insert(key.clone(), design_arc.clone());

        Ok((key, design_arc))
    }

    /// Retrieves a design from the registry or loads it without Yosys preprocessing.
    #[contracts::debug_requires(!module_name.is_empty())]
    pub fn get_or_load_design_raw(
        &self,
        design_path: &str,
        module_name: &str,
    ) -> Result<(DriverKey, Arc<DesignContainer>), DriverError> {
        let absolute_path = self.resolve_path(Path::new(design_path));
        let key = DriverKey::new(&absolute_path, module_name.to_string());

        if let Some(design) = self.check_registry(&key) {
            return Ok((key, design));
        }

        let yosys_module = YosysModule::new(&absolute_path.display().to_string(), module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = yosys_module
            .import_design_raw()
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        let design_arc = Arc::new(DesignContainer::build(design));
        self.registry
            .write()
            .unwrap()
            .insert(key.clone(), design_arc.clone());

        Ok((key, design_arc))
    }

    /// Retrieves a design from the registry if it exists.
    pub fn get_design(&self, key: &DriverKey) -> Option<Arc<DesignContainer>> {
        self.check_registry(key)
    }

    /// Retrieves a design by path and module name if it exists.
    pub fn get_design_by_path<P: AsRef<Path>>(
        &self,
        path: P,
        module_name: &str,
    ) -> Option<Arc<DesignContainer>> {
        let key = DriverKey::new(path, module_name.to_string());
        self.get_design(&key)
    }

    /// Creates an execution context containing the specified designs.
    #[contracts::debug_requires(!keys.is_empty(), "Must request at least one design")]
    pub fn create_context(&self, keys: &[DriverKey]) -> Result<Context, DriverError> {
        let mut context = Context::new();
        let registry = self.registry.read().unwrap();

        for key in keys {
            let design = registry.get(key).ok_or_else(|| {
                DriverError::DesignLoading(format!("Design not found for key: {:?}", key))
            })?;
            context.insert(key.clone(), design.clone());
        }

        Ok(context)
    }

    /// Creates an execution context containing a single design.
    pub fn create_context_single(&self, key: &DriverKey) -> Result<Context, DriverError> {
        self.create_context(&[key.clone()])
    }

    /// Returns a copy of all currently loaded designs.
    pub fn get_all_designs(&self) -> HashMap<DriverKey, Arc<DesignContainer>> {
        self.registry.read().unwrap().clone()
    }
}
