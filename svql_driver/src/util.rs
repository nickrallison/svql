use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Stdio,
};

use log::error;

use crate::driver::DesignKey;

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

pub(crate) fn run_yosys_cmd(
    yosys: &Path,
    design_path: &DesignPath,
    module_name: &str,
) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
    let json_temp_file = tempfile::Builder::new()
        .prefix("svql_prjunnamed_")
        .suffix(".json")
        .rand_bytes(4)
        .tempfile()?;

    let mut cmd = std::process::Command::new(yosys);
    cmd.args(get_command_args_slice(
        &design_path,
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn can_load_driver() {
//         let _driver = load_driver_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
//     }
// }
