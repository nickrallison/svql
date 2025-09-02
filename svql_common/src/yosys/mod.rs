use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YosysModule {
    path: DesignPath,
    module: String,
}

impl YosysModule {
    pub fn new(path: &str, module: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // let design_path = DesignPath::new(PathBuf::from(path))?;

        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let design_path = Path::new(path);
        let design_path = if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            workspace.join(design_path)
        };

        let design_path = DesignPath::new(design_path)?;

        Ok(YosysModule {
            path: design_path,
            module: module.to_string(),
        })
    }
    fn design_path(&self) -> &DesignPath {
        &self.path
    }
    pub fn path(&self) -> &Path {
        &self.path.path()
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    fn get_command_args_slice(&self, json_out: &Path, config: &ModuleConfig) -> Vec<String> {
        let read_cmd = match self.path {
            DesignPath::Verilog(_) => "read_verilog",
            DesignPath::Rtlil(_) => "read_rtlil",
            DesignPath::Json(_) => "read_json",
        };

        let mut args = Vec::new();

        // read command
        args.push("-p".to_string());
        args.push(format!(
            "{} {}",
            read_cmd,
            self.design_path().path().display()
        ));
        args.push("-p".to_string());
        args.push(format!("hierarchy -top {}", self.module_name()));

        // chparam
        for (param, value) in &config.params {
            args.push("-p".to_string());
            args.push(format!(
                "chparam -set {} {} {}",
                param,
                value,
                self.module_name()
            ));
        }

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

        for step in &config.other_steps {
            args.push("-p".to_string());
            args.push(step.clone());
        }

        // TODO: Sanitize Steps?

        // write
        args.push("-p".to_string());
        args.push(format!("write_json {}", json_out.display()));

        args
    }

    pub fn import_design(
        &self,
        module_config: &ModuleConfig,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
        let yosys = which::which("yosys").map_err(|_| "yosys not found on path")?;
        self.import_design_yosys(module_config, &yosys)
    }

    pub fn import_design_yosys(
        &self,
        module_config: &ModuleConfig,
        yosys: &Path,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
        let json_temp_file = tempfile::Builder::new()
            .prefix("svql_prjunnamed_")
            .suffix(".json")
            .rand_bytes(4)
            .tempfile()?;

        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let design_path = if self.design_path().path().is_absolute() {
            self.design_path().path().to_path_buf()
        } else {
            workspace.join(self.design_path().path())
        };

        let design_path = design_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize design path: {}", e))?;

        // let design_path = DesignPath::new(design_path)?;

        let mut cmd = std::process::Command::new(yosys);
        cmd.args(self.get_command_args_slice(json_temp_file.path(), module_config))
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

        let mut designs =
            prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;

        let design = designs.remove(self.module_name()).ok_or_else(|| {
            format!(
                "Design not found in Yosys JSON output: {}",
                design_path.display()
            )
        })?;

        Ok(design)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleConfig {
    pub flatten: bool,
    pub params: HashMap<String, String>,
    // after running proc, [flatten], opt_clean
    pub other_steps: Vec<String>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            flatten: false,
            params: HashMap::new(),
            other_steps: vec![],
        }
    }
}
