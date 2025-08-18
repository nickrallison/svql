use log::{error, trace};
use prjunnamed_netlist::{Cell, CellRef, Design};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use crate::cache::{Cache, DesignKey};

#[derive(Debug, Clone)]
pub struct Driver {
    pub(crate) module_name: String,
    pub(crate) design: Arc<Design>,
}

impl Driver {
    pub fn new(
        design: PathBuf,
        module_name: String,
        cache: Option<&mut Cache>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys =
            which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;
        let design = if design.is_absolute() {
            design
        } else {
            workspace.join(design)
        };

        let design =
            DesignPath::new(design).map_err(|e| format!("Failed to create design: {}", e))?;

        Self::new_yosys(design, module_name, cache, yosys)
    }

    pub fn new_yosys(
        design_path: DesignPath,
        module_name: String,
        cache: Option<&mut Cache>,
        yosys: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        trace!(
            "new_yosys called with yosys: {:?}, design: {:?}, module_name: {}",
            yosys, design_path, module_name
        );
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()).into());
        }

        if !design_path.exists() {
            return Err(
                format!("Design file not found at: {}", design_path.path().display()).into(),
            );
        }

        let mut owned_cache = Cache::new();
        let cache = match cache {
            Some(c) => c,
            None => &mut owned_cache,
        };

        let key = DesignKey {
            path: design_path.clone(),
            top: module_name.clone(),
        };

        if cache.get(&key).is_none() {
            let design_new = run_yosys_cmd(&yosys, &design_path, &module_name)?;
            cache.insert(key.clone(), design_new);
        }

        let design = cache
            .get(&key)
            .expect("Design should be in cache after running Yosys");

        let driver = Driver {
            design,
            module_name,
        };

        Ok(driver)
    }

    pub fn design(&self) -> Arc<Design> {
        self.design.clone()
    }
    pub fn design_as_ref(&self) -> &Design {
        self.design.as_ref()
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn get_input_cells<'p>(&'p self) -> Vec<CellRef<'p>> {
        self.design
            .iter_cells()
            .filter(|cell| matches!(cell.get().as_ref(), Cell::Input(_, _)))
            .collect()
    }
    pub fn get_output_cells<'p>(&'p self) -> Vec<CellRef<'p>> {
        self.design
            .iter_cells()
            .filter(|cell| matches!(cell.get().as_ref(), Cell::Output(_, _)))
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_path_new() {
        let verilog_path = PathBuf::from("examples/patterns/basic/ff/sdffe.v");
        let rtlil_path = PathBuf::from("examples/patterns/basic/ff/sdffe.il");
        let json_path = PathBuf::from("examples/patterns/basic/ff/sdffe.json");
        let unsupported_path = PathBuf::from("examples/patterns/basic/ff/sdffe.txt");

        assert!(DesignPath::new(verilog_path).is_ok());
        assert!(DesignPath::new(rtlil_path).is_ok());
        assert!(DesignPath::new(json_path).is_ok());
        assert!(DesignPath::new(unsupported_path).is_err());
    }

    #[test]
    fn test_run_yosys_cmd() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let design_file = workspace.join("examples/patterns/basic/ff/sdffe.v");
        let yosys = which::which("yosys")
            .map_err(|e| format!("Failed to find yosys binary: {}", e))
            .expect("Yosys binary not found");
        let design_path = DesignPath::new(design_file).unwrap();
        let module_name = "sdffe";

        let result = run_yosys_cmd(&yosys, &design_path, module_name);
        assert!(
            result.is_ok(),
            "expected Ok but got Err: {}",
            result.err().unwrap()
        );
    }

    #[test]
    fn test_driver_new() {
        let design_path = PathBuf::from("examples/patterns/basic/ff/sdffe.v");
        let driver = Driver::new(design_path, "sdffe".into(), None);
        assert!(driver.is_ok());
        let driver = driver.unwrap();
        assert_eq!(driver.module_name(), "sdffe");
    }
}