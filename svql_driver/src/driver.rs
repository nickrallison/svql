use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use prjunnamed_netlist::Design;
use thiserror::Error;

use crate::{Context, DriverKey};

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("Failed to find yosys binary: {0}")]
    YosysNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Design loading error: {0}")]
    DesignLoading(String),
}

#[derive(Debug, Clone)]
pub struct Driver {
    registry: Arc<RwLock<HashMap<DriverKey, Arc<Design>>>>,
    yosys_path: PathBuf,
    root_path: PathBuf,
}

impl Driver {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, DriverError> {
        let yosys = which::which("yosys").map_err(|e| DriverError::YosysNotFound(e.to_string()))?;

        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys_path: yosys,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    pub fn new_workspace() -> Result<Self, DriverError> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::new(workspace)
    }

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

    /// Load a design and store it in the registry
    pub fn load_design<P: AsRef<Path>>(
        &self,
        design_path: P,
        module_name: String,
    ) -> Result<DriverKey, DriverError> {
        let design_path = design_path.as_ref();
        let absolute_path = if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            self.root_path.join(design_path)
        };

        // Check if already loaded
        let key = DriverKey::new(&absolute_path, module_name.clone());
        {
            let registry = self.registry.read().unwrap();
            if registry.contains_key(&key) {
                return Ok(key);
            }
        }

        // Load the design
        let design = self.load_design_from_path(&absolute_path, &module_name)?;

        // Store in registry
        {
            let mut registry = self.registry.write().unwrap();
            registry.insert(key.clone(), Arc::new(design));
        }

        Ok(key)
    }

    /// Get a design from the registry, loading it if necessary
    pub fn get_or_load_design<P: AsRef<Path>>(
        &self,
        design_path: P,
        module_name: String,
    ) -> Result<(DriverKey, Arc<Design>), DriverError> {
        let design_path = design_path.as_ref();
        let absolute_path = if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            self.root_path.join(design_path)
        };

        let key = DriverKey::new(&absolute_path, module_name.clone());

        // Try to get from registry first
        {
            let registry = self.registry.read().unwrap();
            if let Some(design) = registry.get(&key) {
                return Ok((key, design.clone()));
            }
        }

        // Load and store
        let design = self.load_design_from_path(&absolute_path, &module_name)?;
        let design_arc = Arc::new(design);

        {
            let mut registry = self.registry.write().unwrap();
            registry.insert(key.clone(), design_arc.clone());
        }

        Ok((key, design_arc))
    }

    /// Get a design from the registry (returns None if not loaded)
    pub fn get_design(&self, key: &DriverKey) -> Option<Arc<Design>> {
        let registry = self.registry.read().unwrap();
        registry.get(key).cloned()
    }

    /// Get a design by path and module name
    pub fn get_design_by_path<P: AsRef<Path>>(
        &self,
        path: P,
        module_name: &str,
    ) -> Option<Arc<Design>> {
        let key = DriverKey::new(path, module_name.to_string());
        self.get_design(&key)
    }

    /// Create a context with the specified designs
    pub fn create_context(&self, keys: &[DriverKey]) -> Result<Context, DriverError> {
        let mut context = Context::new();
        let registry = self.registry.read().unwrap();

        for key in keys {
            if let Some(design) = registry.get(key) {
                context.insert(key.clone(), design.clone());
            } else {
                return Err(DriverError::DesignLoading(format!(
                    "Design not found for key: {:?}",
                    key
                )));
            }
        }

        Ok(context)
    }

    /// Create a context with a single design
    pub fn create_context_single(&self, key: &DriverKey) -> Result<Context, DriverError> {
        self.create_context(&[key.clone()])
    }

    /// Get all currently loaded designs
    pub fn get_all_designs(&self) -> HashMap<DriverKey, Arc<Design>> {
        let registry = self.registry.read().unwrap();
        registry.clone()
    }

    /// Private helper to load design from path
    fn load_design_from_path(
        &self,
        design_path: &Path,
        module_name: &str,
    ) -> Result<Design, DriverError> {
        // This would use your existing yosys loading logic
        // For brevity, I'm showing a placeholder
        // crate::util::load_design_with_yosys(path, module_name, &self.yosys_path)
        //     .map_err(|e| DriverError::DesignLoading(e.to_string()))
        crate::util::run_yosys_cmd(&self.yosys_path, design_path.to_path_buf(), module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))
    }
}
