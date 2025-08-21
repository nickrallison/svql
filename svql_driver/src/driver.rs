use log::trace;
use prjunnamed_netlist::Design;
use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::util::{DesignPath, run_yosys_cmd};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesignKey {
    path: DesignPath,
    top: String,
}

impl DesignKey {
    pub fn new<P: Into<PathBuf>>(path: P, top: String) -> Result<Self, String> {
        let design_path = DesignPath::new(path.into())
            .map_err(|e| format!("Failed to create design path: {}", e))?;
        Ok(Self {
            path: design_path,
            top,
        })
    }

    fn normalize<P: Into<PathBuf>>(self, root: P) -> Result<Self, Box<dyn std::error::Error>> {
        let root: PathBuf = root.into();
        let abs_root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()?.join(root)
        };

        let key_path: PathBuf = self.path.path().to_owned();
        if key_path.is_absolute() {
            return Ok(self);
        }

        let abs_key_path = abs_root.join(key_path);
        let canonicalized_path = abs_key_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        Ok(Self {
            path: DesignPath::new(canonicalized_path)?,
            top: self.top,
        })
    }
}

impl fmt::Display for DesignKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.path.path().display(), self.top)
    }
}

/// A shared registry of loaded designs keyed by (path, top-module).
#[derive(Clone)]
pub struct Driver {
    registry: Arc<RwLock<HashMap<DesignKey, Arc<Design>>>>,
    yosys: PathBuf,
    root: PathBuf,
}

impl std::fmt::Debug for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.registry.read().unwrap();
        f.debug_struct("Driver")
            .field("entries", &guard.len())
            .field("yosys", &self.yosys)
            .field("root", &self.root)
            .finish()
    }
}

impl Driver {
    // #####################
    // Constructors
    // #####################
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys =
            which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;
        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys,
            root,
        })
    }

    pub fn new_workspace() -> Result<Self, Box<dyn std::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::new(workspace)
    }

    pub fn with_yosys(root: PathBuf, yosys: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()).into());
        }

        let root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()?.join(root)
        };
        let root = root.canonicalize()?;

        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys,
            root,
        })
    }

    // ################
    // Ensure the design is loaded
    // ################

    fn ensure_loaded(&self, key: &DesignKey) -> Result<(), Box<dyn std::error::Error>> {
        {
            if self.registry.read().unwrap().contains_key(&key) {
                return Ok(());
            }
            // drop guard and run yosys
        }

        trace!(
            "Loading design via Yosys: path={} top={}",
            key.path.path().display(),
            &key.top
        );

        let design = run_yosys_cmd(&self.yosys, &key.path, &key.top)?;
        let mut guard = self.registry.write().unwrap();
        guard.insert(key.clone(), Arc::new(design));
        Ok(())
    }

    // ##################
    // Get a design from the registry
    // ##################
    pub fn get<P: Into<PathBuf>>(
        &self,
        path: P,
        module_name: String,
    ) -> Result<Arc<Design>, Box<dyn std::error::Error>> {
        let key = DesignKey::new(path.into(), module_name)?.normalize(&self.root)?;
        self.ensure_loaded(&key)?;

        match self.registry.read() {
            Ok(guard) => {
                let opt_design = guard.get(&key).cloned();
                opt_design.ok_or_else(|| format!("Design not found in registry: {}", key).into())
            }
            Err(e) => {
                return Err(format!("Failed to read registry: {}", e).into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_path_new() {
        let verilog_path = PathBuf::from("examples/patterns/basic/ff/verilog/sdffe.v");
        let rtlil_path = PathBuf::from("examples/patterns/basic/ff/sdffe.il");
        let json_path = PathBuf::from("examples/patterns/basic/ff/sdffe.json");
        let unsupported_path = PathBuf::from("examples/patterns/basic/ff/sdffe.txt");

        assert!(DesignPath::new(verilog_path).is_ok());
        assert!(DesignPath::new(rtlil_path).is_ok());
        assert!(DesignPath::new(json_path).is_ok());
        assert!(DesignPath::new(unsupported_path).is_err());
    }

    #[test]
    fn test_run_yosys_cmd_via_driver() {
        let driver = Driver::new_workspace().unwrap();
        let path = "examples/patterns/basic/ff/verilog/sdffe.v".to_string();
        let module_name = "sdffe".to_string();

        let d = driver
            .get(&path, module_name)
            .expect("design must be present");
        assert!(d.iter_cells().count() > 0);
    }
}
