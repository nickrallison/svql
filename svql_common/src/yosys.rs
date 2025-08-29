use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::Config;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum DesignPath {
    Verilog(PathBuf),
    Rtlil(PathBuf),
    Json(PathBuf),
}

impl DesignPath {
    fn new(path: PathBuf) -> Result<Self, String> {
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
    fn path(&self) -> &Path {
        match self {
            DesignPath::Verilog(p) => p,
            DesignPath::Rtlil(p) => p,
            DesignPath::Json(p) => p,
        }
    }
}

#[contracts::debug_requires(!module_name.is_empty())]
fn get_command_args_slice(
    design: &DesignPath,
    module_name: &str,
    json_out: &Path,
    config: &Config,
) -> Vec<String> {
    let read_cmd = match design {
        DesignPath::Verilog(_) => "read_verilog",
        DesignPath::Rtlil(_) => "read_rtlil",
        DesignPath::Json(_) => "read_json",
    };

    let mut args = Vec::new();

    // read command
    args.push("-p".to_string());
    args.push(format!("{} {}", read_cmd, design.path().display()));
    args.push("-p".to_string());
    args.push(format!("hierarchy -top {}", module_name));

    // proc
    args.push("-p".to_string());
    args.push("proc".to_string());

    // flatten
    if config.flatten {
        args.push("-p".to_string());
        args.push("flatten".to_string());
    }

    // opt clean
    args.push("-p".to_string());
    args.push("opt_clean".to_string());

    // write
    args.push("-p".to_string());
    args.push(format!("write_json {}", json_out.display()));

    args
}

#[contracts::debug_requires(!module_name.is_empty())]
pub fn import_design(
    design_path: PathBuf,
    module_name: &str,
    config: &Config,
) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
    let yosys = which::which("yosys").map_err(|_| "yosys not found on path")?;
    import_design_yosys(&yosys, design_path, module_name, config)
}

#[contracts::debug_requires(yosys.exists(), "yosys path must exist")]
#[contracts::debug_requires(!module_name.is_empty())]
pub fn import_design_yosys(
    yosys: &Path,
    design_path: PathBuf,
    module_name: &str,
    config: &Config,
) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
    let json_temp_file = tempfile::Builder::new()
        .prefix("svql_prjunnamed_")
        .suffix(".json")
        .rand_bytes(4)
        .tempfile()?;

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let design_path = if design_path.is_absolute() {
        design_path
    } else {
        workspace.join(design_path)
    };

    let design_path = design_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize design path: {}", e))?;

    let design_path = DesignPath::new(design_path)?;

    let mut cmd = std::process::Command::new(yosys);
    cmd.args(get_command_args_slice(
        &design_path,
        module_name,
        json_temp_file.path(),
        config,
    ))
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());

    // Use `output()` to read both stdout and stderr concurrently.
    let output = cmd.output()?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let _stdout_str = String::from_utf8_lossy(&output.stdout);
        tracing::event!(
            tracing::Level::ERROR,
            "Yosys failed: status={:?}\n{}",
            output.status,
            stderr_str,
        );
        return Err(format!("Yosys failed: status={:?}\n{}", output.status, stderr_str).into());
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
