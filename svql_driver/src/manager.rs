// svql_driver/src/manager.rs
use std::path::Path;
use std::sync::Arc;

use crate::{
    context::QueryContext,
    key::DesignKey,
    util::{DesignPath, run_yosys_cmd},
};
use prjunnamed_netlist::Design;

#[derive(Debug, Clone)]
pub struct DesignManager {
    registry: Arc<std::sync::RwLock<std::collections::HashMap<DesignKey, Arc<Design>>>>,
    yosys_path: std::path::PathBuf,
    root_path: std::path::PathBuf,
}

impl DesignManager {
    /// Create a new design manager with default yosys detection
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys =
            which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;

        Ok(Self {
            registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            yosys_path: yosys,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    /// Create with explicit yosys path
    pub fn with_yosys<P: AsRef<Path>, Y: AsRef<Path>>(
        root: P,
        yosys: Y,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys_path = yosys.as_ref().to_path_buf();
        if !yosys_path.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys_path.display()).into());
        }

        Ok(Self {
            registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            yosys_path,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    /// Load a design and return a handle to it
    pub fn load_design<P: AsRef<Path>>(
        &self,
        path: P,
        module_name: &str,
    ) -> Result<DesignHandle, Box<dyn std::error::Error>> {
        let key = DesignKey::new(path.as_ref().to_path_buf(), module_name.to_string())?
            .normalize(&self.root_path)?;

        self.ensure_loaded(&key)?;

        Ok(DesignHandle {
            key,
            manager: self.clone(),
        })
    }

    /// Create a query context for orchestrating multiple designs
    pub fn create_context(&self) -> QueryContext {
        QueryContext::new(self.clone())
    }

    /// Preload multiple designs efficiently
    pub fn preload_designs<P: AsRef<Path>, I>(
        &self,
        designs: I,
    ) -> Result<Vec<DesignHandle>, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = (P, String)>,
    {
        let mut handles = Vec::new();
        for (path, module_name) in designs {
            let handle = self.load_design(path, &module_name)?;
            handles.push(handle);
        }
        Ok(handles)
    }

    /// Get design reference by key (internal use)
    pub(crate) fn get_design(&self, key: &DesignKey) -> Option<Arc<Design>> {
        self.registry.read().ok()?.get(key).cloned()
    }

    fn ensure_loaded(&self, key: &DesignKey) -> Result<(), Box<dyn std::error::Error>> {
        {
            if self.registry.read().unwrap().contains_key(key) {
                return Ok(());
            }
        }

        let design = run_yosys_cmd(&self.yosys_path, &key.path, &key.top)?;
        let mut guard = self.registry.write().unwrap();
        guard.insert(key.clone(), Arc::new(design));
        Ok(())
    }
}

/// Handle to a loaded design that can be used in queries
#[derive(Debug, Clone)]
pub struct DesignHandle {
    key: DesignKey,
    manager: DesignManager,
}

impl DesignHandle {
    pub fn path(&self) -> &std::path::Path {
        self.key.path.path()
    }

    pub fn module_name(&self) -> &str {
        &self.key.top
    }

    /// Get the actual design reference (for internal use)
    pub(crate) fn get_design(&self) -> Option<Arc<Design>> {
        self.manager.get_design(&self.key)
    }
}
