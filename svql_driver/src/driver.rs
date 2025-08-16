use log::{error, trace};
use std::{path::{Path, PathBuf}, process::Stdio, sync::{Arc, RwLock}};

use crate::{config::Config, read_input_to_design, subgraph::SubgraphMatch};

#[derive(Debug)]
pub struct Driver {
    design: Arc<prjunnamed_netlist::Design>,
    module_name: String,
}

impl Driver {
    pub fn new(design: PathBuf, module_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys = which::which("yosys")
            .map_err(|e| format!("Failed to find yosys binary: {}", e))?;
        let design = if design.is_absolute() {
            design
        } else {
            workspace.join(design)
        };

        let design = DesignPath::new(design).map_err(|e| format!("Failed to create design: {}", e))?;

        Self::new_yosys(design, module_name, yosys)
    }

    pub fn new_yosys(design: DesignPath, module_name: String, yosys: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        trace!("new_yosys called with yosys: {:?}, design: {:?}, module_name: {}", yosys, design, module_name);
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()).into());
        }

        if !design.exists() {
            return Err(format!("Design file not found at: {}", design.path().display()).into());
        }


        let json_temp_file = tempfile::Builder::new()
            .prefix("svql_prjunnamed_")
            .suffix(".json")
            .rand_bytes(4)
            .tempfile()?;

        let mut cmd = std::process::Command::new(&yosys);
        cmd.args(get_command_args_slice(&design, &module_name, &json_temp_file.path()));
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());


        let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");
        let exit_status = yosys_process.wait().expect("Failed to wait for yosys process");

        if !exit_status.success() {
            let mut stderr = yosys_process.stderr.take().expect("Failed to capture stderr");
            let mut stderr_buf = Vec::new();
            use std::io::Read;
            stderr.read_to_end(&mut stderr_buf).expect("Failed to read stderr");
            let stderr_str = String::from_utf8_lossy(&stderr_buf);
            error!("Yosys process failed with status: {:?}\nStderr: {}", exit_status, stderr_str);
            return Err(format!("Yosys process failed with status: {:?}\nStderr: {}", exit_status, stderr_str).into());
        }

        let design = read_input_to_design(None, json_temp_file.path().to_string_lossy().to_string())
            .map_err(|e| format!("Failed to read input design from Yosys output: {}", e))?;

        let driver = Driver {
            design: Arc::new(design),
            module_name,
        };

        Ok(driver)
    }

    

    pub fn query<'p>(&self, cfg: &Config) -> Vec<SubgraphMatch> {
        trace!("ProcDriver::query called with config: {:?}", cfg);
        todo!()
    }
}


#[derive(Debug, Clone)]
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
            Err(format!("Unsupported design file extension: {:?}", path.extension()))
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

fn get_command(yosys: &Path, design: &DesignPath, module_name: &str, json_out: &Path) -> String {
    let args = get_command_args_slice(design, module_name, json_out);
    let args = args.into_iter().map(|s| 
        match s.contains(" ") {
            true => format!("\"{}\"", s),
            false => s,
        }
    ).collect::<Vec<_>>();
    let args = args.join(" ");
    format!("{} {}", yosys.display(), args)
}
