use log::{error, trace};
use svql_common::config::ffi::SvqlRuntimeConfig;
use std::{path::{self, Path, PathBuf}, process::{Child, Stdio}};

use crate::{net::{NetDriver, SvqlDriverNetError}, DriverIterator};

#[derive(Debug)]
pub struct ProcDriver {
    child: Child,
    driver: NetDriver,
    port: u16,

    // ###
    yosys: PathBuf,
    svql_driver: PathBuf,
    design: Design,
    module_name: String,
}

#[derive(Debug, Clone)]
pub enum Design {
    Verilog(PathBuf),
    Rtlil(PathBuf),
}

impl Design {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if path.extension().and_then(|s| s.to_str()) == Some("v") {
            Ok(Design::Verilog(path))
        } else if path.extension().and_then(|s| s.to_str()) == Some("il") {
            Ok(Design::Rtlil(path))
        } else {
            Err(format!("Unsupported design file extension: {:?}", path.extension()))
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            Design::Verilog(p) => p,
            Design::Rtlil(p) => p,
        }
    }
    pub fn exists(&self) -> bool {
        self.path().exists()
    }
}

impl ProcDriver {

    pub fn new(design: PathBuf, module_name: String) -> Result<Self, String> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys = workspace.join("yosys/yosys");
        let svql_driver = workspace.join("build/svql_driver/libsvql_driver.so");
        let design = if design.is_absolute() {
            design
        } else {
            workspace.join(design)
        };

        let design = Design::new(design).map_err(|e| format!("Failed to create design: {}", e))?;

        Self::new_yosys(yosys, svql_driver, design, module_name)
    }

    pub fn new_yosys(yosys: PathBuf, svql_driver: PathBuf, design: Design, module_name: String) -> Result<Self, String> {
        trace!("new_yosys called with yosys: {:?}, svql_driver: {:?}, design: {:?}, module_name: {}", yosys, svql_driver, design, module_name);
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()));
        }
        if !svql_driver.exists() {
            return Err(format!("SVQL driver not found at: {}", svql_driver.display()));
        }
        if !design.exists() {
            return Err(format!("Design file not found at: {}", design.path().display()));
        }

        let openport = openport::pick_unused_port(15000..25000).expect("Failed to find open port");

        let mut cmd = std::process::Command::new(&yosys);
        cmd.args(get_command_args_slice(&svql_driver, &design, &module_name, openport));
        // Suppress direct stdout/stderr of the Yosys process; we'll capture it via wait_until_ready
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");

        // Wait for yosys to be ready
        let ready_marker = "SVQL DRIVER server - TCP acceptor created - port";
        let timeout_secs: u64 = 10;

        wait_until_ready(&mut yosys_process, ready_marker, timeout_secs)?;

        let driver = NetDriver::new(format!("localhost:{}", openport));

        let yosys_proc = ProcDriver {
            child: yosys_process,
            port: openport,
            driver,

            yosys,
            svql_driver,
            design,
            module_name,
        };
        Ok(yosys_proc)
    }

    pub fn get_command(&self) -> String {
        get_command(&self.yosys, &self.svql_driver, &self.design, &self.module_name, self.port)
    }

    pub fn kill(mut self) -> Result<(), String> {
        self.child.kill().map_err(|e| format!("Failed to kill yosys process: {}", e))
    }

    pub fn query(&self, cfg: &SvqlRuntimeConfig) -> Result<DriverIterator, SvqlDriverNetError> {
        trace!("ProcDriver::query called with config: {:?}", cfg);
        self.driver.query(cfg)
    }
}


fn get_command_args_slice(svql_driver: &Path, design: &Design, module_name: &str, port: u16) -> Vec<String> {

    let read_cmd = match design {
        Design::Verilog(_) => "read_verilog",
        Design::Rtlil(_) => "read_rtlil",
    };

    vec![
        "-m".to_string(),
        format!("{}", svql_driver.display()),
        "-p".to_string(),
        format!("{} {}", read_cmd, design.path().display()),
        "-p".to_string(),
        format!("hierarchy -top {}", module_name),
        "-p".to_string(),
        "proc".to_string(),
        "-p".to_string(),
        format!("svql_driver -net -port {}", port),
    ]
}

fn get_command(yosys: &Path, svql_driver: &Path, design: &Design, module_name: &str, port: u16) -> String {
    let args = get_command_args_slice(svql_driver, design, module_name, port);
    let args = args.into_iter().map(|s| 
        match s.contains(" ") {
            true => format!("\"{}\"", s),
            false => s,
        }
    ).collect::<Vec<_>>();
    let args = args.join(" ");
    format!("{} {}", yosys.display(), args)
}

fn wait_until_ready(
    yosys_process: &mut Child,
    ready_marker: &str,
    timeout_secs: u64,
) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    if let Some(stdout) = yosys_process.stdout.take() {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = tx.send(l);
                } else {
                    break;
                }
            }
        });
    }

    let timeout = std::time::Duration::from_secs(timeout_secs as u64);
    let start = std::time::Instant::now();
    let mut ready = false;
    let mut recent: std::collections::VecDeque<String> =
        std::collections::VecDeque::with_capacity(50);

    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                let is_match = line.contains(&ready_marker);
                trace!(target: "wait_until_ready", "line: '{}' | match: {}", line, is_match);
                if recent.len() == recent.capacity() {
                    recent.pop_front();
                }
                recent.push_back(line.clone());
                if is_match {
                    ready = true;
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout waiting for a line, break and report error
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Channel closed, but keep waiting until timeout
                // disconnected = true; // unused
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    if !ready {
        error!(target: "wait_until_ready", "Timed out waiting for yosys to be ready. Expected line containing: '{}'. Recent output: {}", ready_marker, recent.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
        return Err(format!(
            "Timed out waiting for yosys to be ready. Expected line containing: '{}'.\nRecent output:\n{}",
            ready_marker,
            recent.into_iter().collect::<Vec<_>>().join("\n")
        ));
    }
    Ok(())
}