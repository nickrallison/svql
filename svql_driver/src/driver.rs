use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, path::Path};

use prjunnamed_netlist::Design;

#[derive(Debug, Clone)]
pub struct Driver {
    registry: Arc<RwLock<HashMap<DriverKey, Arc<Design>>>>,
    yosys_path: std::path::PathBuf,
    root_path: std::path::PathBuf,
}

impl Driver {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys =
            which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;

        Ok(Self {
            registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            yosys_path: yosys,
            root_path: std::fs::canonicalize(root.as_ref())?,
        })
    }

    pub fn new_workspace() -> Result<Self, Box<dyn std::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self::new(workspace)
    }

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

    // #####################
    // methods to manage designs
    // #####################

    // TBD
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriverKey {
    // TBD
}
