use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use prjunnamed_netlist::Design;
use svql_common::YosysModule;
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

    // #[contracts::debug_ensures(ret.as_ref().map(|o| o.root_path.is_absolute()).unwrap_or(true), "Root path must be absolute")]
    // #[contracts::debug_ensures(ret.as_ref().map(|o| o.yosys_path.exists()).unwrap_or(true), "Custom yosys path must exist")]
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
    #[contracts::debug_requires(!module_name.is_empty())]
    pub fn load_design<P: AsRef<Path>>(
        &self,
        design_path: P,
        module_name: String,
        module_config: &svql_common::ModuleConfig,
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
                tracing::event!(tracing::Level::DEBUG, "Design already loaded: {:?}", key);
                return Ok(key);
            }
        }

        // Load the design
        tracing::event!(
            tracing::Level::INFO,
            "Loading design: {} ({})",
            absolute_path.display(),
            module_name
        );
        let yosys_module = YosysModule::new(&absolute_path.display().to_string(), &module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = yosys_module
            .import_design(module_config)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        // Store in registry
        {
            let mut registry = self.registry.write().unwrap();
            registry.insert(key.clone(), Arc::new(design));
            tracing::event!(
                tracing::Level::DEBUG,
                "Design stored in registry: {:?}",
                &key
            );
        }

        Ok(key)
    }

    /// Get a design from the registry, loading it if necessary
    #[contracts::debug_requires(!module_name.is_empty())]
    pub fn get_or_load_design(
        &self,
        design_path: &str,
        module_name: &str,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<(DriverKey, Arc<Design>), DriverError> {
        let design_path = Path::new(design_path);
        let absolute_path = if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            self.root_path.join(design_path)
        };

        let key = DriverKey::new(&absolute_path, module_name.to_string());

        // Try to get from registry first
        {
            let registry = self.registry.read().unwrap();
            if let Some(design) = registry.get(&key) {
                tracing::event!(tracing::Level::DEBUG, "Design found in registry: {:?}", key);
                return Ok((key, design.clone()));
            }
        }

        // Load and store
        tracing::event!(
            tracing::Level::INFO,
            "Loading design: {} ({})",
            absolute_path.display(),
            module_name
        );

        let yosys_module = YosysModule::new(&absolute_path.display().to_string(), &module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;
        let design = yosys_module
            .import_design(module_config)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        let design_arc = Arc::new(design);

        {
            let mut registry = self.registry.write().unwrap();
            registry.insert(key.clone(), design_arc.clone());
            tracing::event!(
                tracing::Level::DEBUG,
                "Design stored in registry: {:?}",
                key
            );
        }

        Ok((key, design_arc))
    }

    /// Get a design from the registry (returns None if not loaded)
    pub fn get_design(&self, key: &DriverKey) -> Option<Arc<Design>> {
        let registry = self.registry.read().unwrap();
        let result = registry.get(key).cloned();
        if result.is_some() {
            tracing::event!(
                tracing::Level::DEBUG,
                "Design retrieved from registry: {:?}",
                key
            );
        } else {
            tracing::event!(
                tracing::Level::WARN,
                "Design not found in registry: {:?}",
                key
            );
        }
        result
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
    #[contracts::debug_requires(!keys.is_empty(), "Must request at least one design")]
    pub fn create_context(&self, keys: &[DriverKey]) -> Result<Context, DriverError> {
        let mut context = Context::new();
        let registry = self.registry.read().unwrap();

        for key in keys {
            if let Some(design) = registry.get(key) {
                context.insert(key.clone(), design.clone());
                tracing::event!(tracing::Level::DEBUG, "Design added to context: {:?}", key);
            } else {
                tracing::event!(
                    tracing::Level::WARN,
                    "Design not found in registry: {:?}",
                    key
                );
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
        tracing::event!(
            tracing::Level::DEBUG,
            "Creating context with single design: {:?}",
            key
        );
        self.create_context(&[key.clone()])
    }

    /// Get all currently loaded designs
    pub fn get_all_designs(&self) -> HashMap<DriverKey, Arc<Design>> {
        let registry = self.registry.read().unwrap();
        tracing::event!(
            tracing::Level::DEBUG,
            "Retrieved all designs from registry (count: {})",
            registry.len()
        );
        registry.clone()
    }

    /// Private helper to load design from path
    fn load_design_from_path(
        &self,
        design_path: &str,
        module_name: &str,
        module_config: &svql_common::ModuleConfig,
    ) -> Result<Design, DriverError> {
        tracing::event!(
            tracing::Level::DEBUG,
            "Loading design from path: {} ({})",
            design_path,
            module_name
        );
        let yosys_module = YosysModule::new(design_path, module_name)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        let result = yosys_module
            .import_design(module_config)
            .map_err(|e| DriverError::DesignLoading(e.to_string()))?;

        Ok(result)

        // svql_common::import_design_yosys(
        //     &self.yosys_path,
        //     design_path.to_path_buf(),
        //     module_name,
        //     config,
        // )
        // .map_err(|e| DriverError::DesignLoading(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Driver, DriverError, DriverKey};
    use tracing_subscriber;

    fn init_test_logger() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn driver_create_workspace() {
        init_test_logger();
        let d = Driver::new_workspace().expect("workspace driver");
        // registry should be empty initially
        assert_eq!(d.get_all_designs().len(), 0);
    }

    #[test]
    fn driver_create_context_missing_key() {
        init_test_logger();
        let d = Driver::new_workspace().expect("workspace driver");
        // Make a key that won't be in the registry
        let k = DriverKey::new("nonexistent.v", "missing_top".to_string());
        let err = d.create_context(&[k]).unwrap_err();
        match err {
            DriverError::DesignLoading(msg) => assert!(msg.contains("Design not found")),
            _ => panic!("unexpected error variant"),
        }
    }
}
