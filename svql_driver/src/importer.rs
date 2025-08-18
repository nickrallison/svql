use std::fs::File;
use std::path::PathBuf;
use std::process::Stdio;

use crate::cache::{Cache, DesignKey};
use crate::driver::{DesignPath, Driver};

pub trait Importer {
    fn import(
        &self,
        design: &DesignPath,
        top: &str,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>>;
}

pub struct YosysImporter {
    pub yosys: PathBuf,
    pub passes: Vec<String>, // e.g. ["proc", "flatten", "opt_clean"]
}

impl YosysImporter {
    pub fn new(yosys: PathBuf) -> Self {
        Self {
            yosys,
            passes: vec!["proc".into(), "flatten".into(), "opt_clean".into()],
        }
    }
}

impl Importer for YosysImporter {
    fn import(
        &self,
        design: &DesignPath,
        top: &str,
    ) -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
        if !self.yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", self.yosys.display()).into());
        }
        if !design.exists() {
            return Err(format!("Design file not found at: {}", design.path().display()).into());
        }

        let json_temp_file = tempfile::Builder::new()
            .prefix("svql_prjunnamed_")
            .suffix(".json")
            .rand_bytes(4)
            .tempfile()?;

        let read_cmd = match design {
            DesignPath::Verilog(_) => "read_verilog",
            DesignPath::Rtlil(_) => "read_rtlil",
            DesignPath::Json(_) => "read_json",
        };

        let mut cmd = std::process::Command::new(&self.yosys);
        cmd.arg("-p")
            .arg(format!("{} {}", read_cmd, design.path().display()))
            .arg("-p")
            .arg(format!("hierarchy -top {}", top))
            .arg("-p")
            .arg(self.passes.join("; "))
            .arg("-p")
            .arg(format!("write_json {}", json_temp_file.path().display()));

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
            return Err(format!(
                "Yosys process failed with status: {:?}\nStderr: {}",
                exit_status, stderr_str
            )
            .into());
        }

        let designs = prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;
        if designs.len() != 1 {
            return Err("can only convert single-module Yosys JSON to Unnamed IR".into());
        }
        let design = designs.into_values().next().unwrap();
        Ok(design)
    }
}

pub struct DriverBuilder<I: Importer> {
    importer: I,
    cache: Option<Cache>,
}

impl<I: Importer> DriverBuilder<I> {
    pub fn new(importer: I) -> Self {
        Self {
            importer,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn build(
        mut self,
        path: DesignPath,
        top: String,
    ) -> Result<Driver, Box<dyn std::error::Error>> {
        let mut cache = self.cache.take().unwrap_or_default();
        let key = DesignKey {
            path: path.clone(),
            top: top.clone(),
        };

        let design = if let Some(d) = cache.get(&key) {
            d
        } else {
            let d = self.importer.import(&path, &top)?;
            cache.insert(key.clone(), d);
            cache
                .get(&key)
                .expect("design must be present after insert")
        };

        Ok(Driver {
            module_name: top,
            design,
        })
    }
}
