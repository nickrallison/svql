use log::{error, trace};
use prjunnamed_netlist::Design;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, RwLock},
};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesignPath {
    Verilog(PathBuf),
    Rtlil(PathBuf),
    Json(PathBuf),
}

impl DesignPath {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if path.extension().and_then(|s| s.to_str()) == Some("v") {
            Ok(DesignPath::Verilog(path))
        } else if path.extension().and_then(|s| s.to_str()) == Some("il") {
            Ok(DesignPath::Rtlil(path))
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            Ok(DesignPath::Json(path))
        } else {
            Err(format!(
                "Unsupported design file extension: {:?}",
                path.extension()
            ))
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            DesignPath::Verilog(p) => p,
            DesignPath::Rtlil(p) => p,
            DesignPath::Json(p) => p,
        }
    }
    pub fn exists(&self) -> bool {
        self.path().exists()
    }
}

fn get_command_args_slice(design: &DesignPath, module_name: &str, json_out: &Path) -> Vec<String> {
    let read_cmd = match design {
        DesignPath::Verilog(_) => "read_verilog",
        DesignPath::Rtlil(_) => "read_rtlil",
        DesignPath::Json(_) => "read_json",
    };

    vec![
        "-p".to_string(),
        format!("{} {}", read_cmd, design.path().display()),
        "-p".to_string(),
        format!("hierarchy -top {}", module_name),
        "-p".to_string(),
        "proc; flatten; opt_clean".to_string(),
        "-p".to_string(),
        format!("write_json {}", json_out.display()),
    ]
}

fn run_yosys_cmd(
    yosys: &Path,
    design: &DesignPath,
    module_name: &str,
) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
    let json_temp_file = tempfile::Builder::new()
        .prefix("svql_prjunnamed_")
        .suffix(".json")
        .rand_bytes(4)
        .tempfile()?;

    let mut cmd = std::process::Command::new(yosys);
    cmd.args(get_command_args_slice(
        design,
        module_name,
        json_temp_file.path(),
    ));
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");
    let exit_status = yosys_process
        .wait()
        .expect("Failed to wait for yosys process");

    if !exit_status.success() {
        let mut stderr = yosys_process
            .stderr
            .take()
            .expect("Failed to capture stderr");
        let mut stderr_buf = Vec::new();
        use std::io::Read;
        stderr
            .read_to_end(&mut stderr_buf)
            .expect("Failed to read stderr");
        let stderr_str = String::from_utf8_lossy(&stderr_buf);
        error!(
            "Yosys process failed with status: {:?}\nStderr: {}",
            exit_status, stderr_str
        );
        return Err(format!(
            "Yosys process failed with status: {:?}\nStderr: {}",
            exit_status, stderr_str
        )
        .into());
    }

    let designs = prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;
    assert_eq!(
        designs.len(),
        1,
        "can only convert single-module Yosys JSON to Unnamed IR"
    );
    let design = designs.into_values().next().unwrap();

    Ok(design)
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

    /// Ensure a design exists in the registry by (path, module_name). Returns the key.
    pub fn ensure_loaded_with_top(
        &self,
        path: PathBuf,
        module_name: String,
    ) -> Result<DesignKey, Box<dyn std::error::Error>> {
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
            let guard = self.registry.read().unwrap();
            if guard.contains_key(&key) {
                return Ok(key);
            }
        }

        trace!(
            "Loading design via Yosys: path={} top={}",
            design_path.path().display(),
            module_name
        );

        let design = run_yosys_cmd(&self.yosys, &design_path, &module_name)?;
        let mut guard = self.registry.write().unwrap();
        guard.insert(key.clone(), Arc::new(design));
        Ok(key)
    }

    /// Ensure a design exists by path; module name is file stem.
    pub fn ensure_loaded(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<DesignKey, Box<dyn std::error::Error>> {
        let p = path.as_ref();
        let module_name = p
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Failed to extract module name from path: {}", p.display()))?
            .to_string();
        self.ensure_loaded_with_top(p.to_path_buf(), module_name)
    }

    /// Get an Arc<Design> for a key (clone Arc for cheap sharing).
    pub fn get(&self, key: &DesignKey) -> Option<Arc<Design>> {
        let guard = self.registry.read().ok()?;
        guard.get(key).cloned()
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
        let driver = Driver::new().unwrap();
        let key = driver
            .ensure_loaded("examples/patterns/basic/ff/verilog/sdffe.v")
            .unwrap();
        let d = driver.get(&key).expect("design must be present");
        assert!(d.iter_cells().count() > 0);
    }
}
