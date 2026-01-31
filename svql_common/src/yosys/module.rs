//! Wrapper for Yosys modules and execution.

use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::yosys::{DesignPath, ModuleConfig};

/// Represents a specific module within a design file to be processed by Yosys.
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
    /// Creates a new `YosysModule` reference.
    /// Resolves relative paths against the workspace root.
    pub fn new(path: &str, module: &str) -> Result<Self, Box<dyn core::error::Error>> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let design_path = Path::new(path);
        let design_path = if design_path.is_absolute() {
            design_path.to_path_buf()
        } else {
            workspace.join(design_path)
        };

        let design_path = DesignPath::new(design_path)?;

        Ok(Self {
            path: design_path,
            module: module.to_owned(),
        })
    }

    /// Returns the categorized design path.
    #[must_use] 
    pub const fn design_path(&self) -> &DesignPath {
        &self.path
    }

    /// Returns the filesystem path.
    #[must_use] 
    pub fn path(&self) -> &Path {
        self.path.path()
    }

    /// Returns the top module name.
    #[must_use] 
    pub fn module_name(&self) -> &str {
        &self.module
    }

    /// Generates the sequence of Yosys commands required to process the design.
    fn generate_yosys_args(
        &self,
        output_path: &Path,
        config: &ModuleConfig,
        output_format: OutputFormat,
    ) -> Vec<String> {
        let mut args = Vec::new();

        args.push("-p".to_owned());
        if config.verific {
            args.push(format!("verific -sv {}", self.path().display()));
        } else {
            args.push(format!(
                "{} {}",
                self.design_path().read_command(),
                self.path().display()
            ));
        }

        args.push("-p".to_owned());
        args.push(format!("hierarchy -top {}", self.module_name()));

        for (param, value) in &config.params {
            args.push("-p".to_owned());
            args.push(format!(
                "chparam -set {} {} {}",
                param,
                value,
                self.module_name()
            ));
        }

        args.push("-p".to_owned());
        args.push("proc".to_owned());

        args.push("-p".to_owned());
        args.push("chformal -remove".to_owned());

        args.push("-p".to_owned());
        args.push("memory".to_owned());

        if config.flatten {
            args.push("-p".to_owned());
            args.push("flatten".to_owned());
        }

        if config.opt {
            args.push("-p".to_owned());
            args.push("opt".to_owned());
        }

        if config.opt_clean {
            args.push("-p".to_owned());
            args.push("opt_clean".to_owned());
        }

        for step in &config.other_steps {
            args.push("-p".to_owned());
            args.push(step.clone());
        }

        args.push("-p".to_owned());
        args.push("delete t:\\$verific$*".to_owned());

        args.push("-p".to_owned());
        let write_cmd = match output_format {
            OutputFormat::Json => format!("write_json {}", output_path.display()),
            OutputFormat::Rtlil => format!("write_rtlil {}", output_path.display()),
        };
        args.push(write_cmd);

        args
    }

    /// Executes the Yosys process with the provided arguments.
    fn execute_yosys(
        &self,
        args: Vec<String>,
        yosys_binary: &Path,
    ) -> Result<(), Box<dyn core::error::Error>> {
        let output = Command::new(yosys_binary)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Yosys execution failed: {}\nSTDOUT: {}\nSTDERR: {}",
                output.status, stdout, stderr
            )
            .into());
        }

        Ok(())
    }

    /// Imports the design into the internal netlist format by invoking Yosys.
    /// Automatically locates the yosys binary in the system PATH.
    pub fn import_design(
        &self,
        module_config: &ModuleConfig,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn core::error::Error>> {
        let yosys = which::which("yosys").map_err(|_| "yosys binary not found in PATH")?;
        self.import_design_yosys(module_config, &yosys)
    }

    /// Imports the design using a specific Yosys binary path.
    pub fn import_design_yosys(
        &self,
        module_config: &ModuleConfig,
        yosys_binary: &Path,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn core::error::Error>> {
        let json_temp = tempfile::Builder::new()
            .prefix("svql_")
            .suffix(".json")
            .tempfile()?;

        let args = self.generate_yosys_args(json_temp.path(), module_config, OutputFormat::Json);
        self.execute_yosys(args, yosys_binary)?;

        self.parse_json_output(json_temp.path())
    }

    /// Imports the design without any preprocessing by Yosys.
    pub fn import_design_raw(
        &self,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn core::error::Error>> {
        self.parse_json_output(self.path())
    }

    /// Internal helper to parse Yosys JSON output and extract the target module.
    fn parse_json_output(
        &self,
        path: &Path,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn core::error::Error>> {
        let mut designs = prjunnamed_yosys_json::import(None, &mut File::open(path)?)?;

        designs.remove(self.module_name()).ok_or_else(|| {
            format!(
                "Module '{}' not found in Yosys output for {}",
                self.module_name(),
                path.display()
            )
            .into()
        })
    }

    /// Writes the processed design in RTLIL format to the specified path.
    pub fn write_rtlil_to_path(
        &self,
        config: &ModuleConfig,
        yosys_binary: &Path,
        rtlil_out: &Path,
    ) -> Result<(), Box<dyn core::error::Error>> {
        let args = self.generate_yosys_args(rtlil_out, config, OutputFormat::Rtlil);
        self.execute_yosys(args, yosys_binary)
    }

    /// Writes the processed design in RTLIL format to standard output.
    pub fn write_rtlil_to_stdout(
        &self,
        config: &ModuleConfig,
        yosys_binary: &Path,
    ) -> Result<(), Box<dyn core::error::Error>> {
        let rtlil_temp = tempfile::Builder::new()
            .prefix("svql_")
            .suffix(".il")
            .tempfile()?;

        self.write_rtlil_to_path(config, yosys_binary, rtlil_temp.path())?;
        let content = std::fs::read_to_string(rtlil_temp.path())?;
        print!("{content}");

        Ok(())
    }
}
