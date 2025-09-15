use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::yosys::{DesignPath, ModuleConfig};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YosysModule {
    path: DesignPath,
    module: String,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Json,
    Rtlil,
}

impl YosysModule {
    pub fn new(path: &str, module: &str) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn design_path(&self) -> &DesignPath {
        &self.path
    }

    pub fn path(&self) -> &Path {
        self.path.path()
    }

    pub fn module_name(&self) -> &str {
        &self.module
    }

    fn build_yosys_args(
        &self,
        output_path: &Path,
        config: &ModuleConfig,
        output_format: OutputFormat,
    ) -> Vec<String> {
        let mut args = Vec::new();

        // Read command
        args.push("-p".to_string());
        args.push(format!(
            "{} {}",
            self.design_path().read_command(),
            self.path().display()
        ));

        // Hierarchy
        args.push("-p".to_string());
        args.push(format!("hierarchy -top {}", self.module_name()));

        // Parameters
        for (param, value) in &config.params {
            args.push("-p".to_string());
            args.push(format!(
                "chparam -set {} {} {}",
                param,
                value,
                self.module_name()
            ));
        }

        // Process
        args.push("-p".to_string());
        args.push("proc".to_string());

        // Memory
        args.push("-p".to_string());
        args.push("memory".to_string());

        // Flatten
        if config.flatten {
            args.push("-p".to_string());
            args.push("flatten".to_string());
        }

        // Optimize and clean
        args.push("-p".to_string());
        args.push("opt_clean".to_string());

        // Other steps
        for step in &config.other_steps {
            args.push("-p".to_string());
            args.push(step.clone());
        }

        // Write output
        args.push("-p".to_string());
        let write_cmd = match output_format {
            OutputFormat::Json => format!("write_json {}", output_path.display()),
            OutputFormat::Rtlil => format!("write_rtlil {}", output_path.display()),
        };
        args.push(write_cmd);

        args
    }

    fn run_yosys_command(
        &self,
        args: Vec<String>,
        yosys: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let yosys = which::which("yosys").map_err(|_| "yosys not found on path")?;

        let mut cmd = Command::new(yosys);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            tracing::event!(
                tracing::Level::ERROR,
                "Yosys failed: status={:?}\n{}",
                output.status,
                stderr_str,
            );
            return Err(format!("Yosys failed: status={:?}\n{}", output.status, stderr_str).into());
        }

        Ok(())
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

        let args = self.build_yosys_args(json_temp_file.path(), module_config, OutputFormat::Json);
        self.run_yosys_command(args, yosys)?;

        let mut designs =
            prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;

        let design = designs.remove(self.module_name()).ok_or_else(|| {
            format!(
                "Design not found in Yosys JSON output: {}",
                self.path().display()
            )
        })?;

        Ok(design)
    }

    pub fn write_rtlil_to_path(
        &self,
        config: &ModuleConfig,
        yosys: &Path,
        rtlil_out: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = self.build_yosys_args(rtlil_out, config, OutputFormat::Rtlil);

        self.run_yosys_command(args, yosys)
    }

    pub fn write_rtlil_to_stdout(
        &self,
        config: &ModuleConfig,
        yosys: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rtlil_temp_file = tempfile::Builder::new()
            .prefix("svql_rtlil_")
            .suffix(".il")
            .rand_bytes(4)
            .tempfile()?;

        self.write_rtlil_to_path(config, rtlil_temp_file.path(), yosys)?;

        let content = std::fs::read_to_string(rtlil_temp_file.path())?;
        print!("{}", content);

        Ok(())
    }
}
