use log::trace;
use prjunnamed_netlist::Design;
use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::util::{DesignPath, run_yosys_cmd};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesignKey {
    pub path: DesignPath,
    pub top: String,
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
    workspace: PathBuf,
}

impl std::fmt::Debug for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.registry.read().unwrap();
        f.debug_struct("Driver")
            .field("entries", &guard.len())
            .field("yosys", &self.yosys)
            .field("workspace", &self.workspace)
            .finish()
    }
}

impl Driver {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys =
            which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;
        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys,
            workspace,
        })
    }

    pub fn with_yosys(yosys: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()).into());
        }
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Ok(Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            yosys,
            workspace,
        })
    }

    /// Ensure a design exists in the registry by (path, module_name).
    pub fn ensure_loaded_with_top(
        &self,
        path: PathBuf,
        module_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let design_path_buf = if path.is_absolute() {
            path
        } else {
            self.workspace.join(path)
        };
        let design_path = DesignPath::new(design_path_buf.clone())
            .map_err(|e| format!("Failed to create design path: {}", e))?;

        if !design_path.exists() {
            return Err(
                format!("Design file not found at: {}", design_path.path().display()).into(),
            );
        }

        let key = DesignKey {
            path: design_path.clone(),
            top: module_name.clone(),
        };

        {
            let guard = self
                .registry
                .read()
                .map_err(|e| format!("registry read lock poisoned: {}", e))?;
            if guard.contains_key(&key) {
                return Ok(());
            }
        }

        trace!(
            "Loading design via Yosys: path={} top={}",
            design_path.path().display(),
            module_name
        );

        let design = run_yosys_cmd(&self.yosys, &design_path, &module_name)?;
        let mut guard = self
            .registry
            .write()
            .map_err(|e| format!("registry write lock poisoned: {}", e))?;
        guard.insert(key.clone(), Arc::new(design));
        Ok(())
    }

    /// Ensure a design exists by path; module name is file stem.
    pub fn ensure_loaded(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let p = path.as_ref();
        let module_name = p
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Failed to extract module name from path: {}", p.display()))?
            .to_string();
        self.ensure_loaded_with_top(p.to_path_buf(), module_name)
    }

    /// Get an Arc<Design> for a key (clone Arc for cheap sharing).
    pub fn get(&self, key: &DesignKey) -> Result<Arc<Design>, Box<dyn std::error::Error>> {
        // Read, check. If missing, load with a write. Then read again to fetch.
        {
            let guard = self
                .registry
                .read()
                .map_err(|e| format!("registry read lock poisoned: {}", e))?;
            if let Some(design) = guard.get(key) {
                return Ok(design.clone());
            }
        }

        // Not present; load it, then fetch.
        self.ensure_loaded_with_top(key.path.path().to_path_buf(), key.top.clone())?;

        let guard = self
            .registry
            .read()
            .map_err(|e| format!("registry read lock poisoned: {}", e))?;
        guard
            .get(key)
            .cloned()
            .ok_or_else(|| format!("Design not found for key: {:?}", key).into())
    }

    pub fn get_by_path(
        &self,
        path: &Path,
        module_name: &str,
    ) -> Result<Arc<Design>, Box<dyn std::error::Error>> {
        let key = DesignKey {
            path: DesignPath::new(path.to_path_buf())
                .map_err(|e| format!("bad design path: {}", e))?,
            top: module_name.to_string(),
        };
        self.get(&key)
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

        assert!(crate::util::DesignPath::new(verilog_path).is_ok());
        assert!(crate::util::DesignPath::new(rtlil_path).is_ok());
        assert!(crate::util::DesignPath::new(json_path).is_ok());
        assert!(crate::util::DesignPath::new(unsupported_path).is_err());
    }

    #[test]
    fn test_run_yosys_cmd_via_driver() {
        let driver = Driver::new().unwrap();
        driver
            .ensure_loaded("examples/patterns/basic/ff/verilog/sdffe.v")
            .unwrap();

        let key = DesignKey {
            path: crate::util::DesignPath::new(PathBuf::from(
                "examples/patterns/basic/ff/verilog/sdffe.v",
            ))
            .unwrap(),
            top: "sdffe".to_string(),
        };

        let d = driver.get(&key).expect("design must be present");
        assert!(d.iter_cells().count() > 0);
    }
}
