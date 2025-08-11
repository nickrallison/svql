use log::{error, trace};
use std::{path::{Path, PathBuf}, process::{Child, Stdio}};
use driver::Driver;

pub mod driver;

#[derive(Debug)]
pub struct YosysProc {
    child: Child,
    port: u16,

    // ###
    yosys: PathBuf,
    svql_driver: PathBuf,
    design: PathBuf,
    module_name: String,
}

impl YosysProc {
    pub fn new_nonblocking(design: PathBuf, module_name: String) -> Result<Self, String> {
        trace!("new_nonblocking called with design: {:?}, module_name: {}", design, module_name);
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys = workspace.join("yosys/yosys");
        let svql_driver = workspace.join("build/svql_driver/libsvql_driver.so");
        let design = if design.is_absolute() {
            design
        } else {
            workspace.join(design)
        };
    
        Self::new_yosys_nonblocking(yosys, svql_driver, design, module_name)
    }

    pub fn new(design: PathBuf, module_name: String) -> Result<Self, String> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let yosys = workspace.join("yosys/yosys");
        let svql_driver = workspace.join("build/svql_driver/libsvql_driver.so");
        let design = if design.is_absolute() {
            design
        } else {
            workspace.join(design)
        };

        Self::new_yosys(yosys, svql_driver, design, module_name)
    }

    pub fn new_yosys_nonblocking(yosys: PathBuf, svql_driver: PathBuf, design: PathBuf, module_name: String) -> Result<Self, String> {

        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()));
        }
        if !svql_driver.exists() {
            return Err(format!("SVQL driver not found at: {}", svql_driver.display()));
        }
        if !design.exists() {
            return Err(format!("Design file not found at: {}", design.display()));
        }

        let openport = openport::pick_unused_port(15000..25000).expect("Failed to find open port");

        let mut cmd = std::process::Command::new(&yosys);
        cmd.args(get_command_args_slice(&svql_driver, &design, &module_name, openport));
        // Suppress direct stdout/stderr of the Yosys process; we'll capture it via wait_until_ready
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let yosys_process = cmd.spawn().expect("Failed to start yosys process");

        let yosys_proc = YosysProc {
            child: yosys_process,
            port: openport,

            yosys,
            svql_driver,
            design,
            module_name,
        };
        Ok(yosys_proc)
    }

    pub fn new_yosys(yosys: PathBuf, svql_driver: PathBuf, design: PathBuf, module_name: String) -> Result<Self, String> {
        trace!("new_yosys called with yosys: {:?}, svql_driver: {:?}, design: {:?}, module_name: {}", yosys, svql_driver, design, module_name);
        if !yosys.exists() {
            return Err(format!("Yosys binary not found at: {}", yosys.display()));
        }
        if !svql_driver.exists() {
            return Err(format!("SVQL driver not found at: {}", svql_driver.display()));
        }
        if !design.exists() {
            return Err(format!("Design file not found at: {}", design.display()));
        }

        let openport = openport::pick_unused_port(15000..25000).expect("Failed to find open port");

        let mut cmd = std::process::Command::new(&yosys);
        cmd.args(get_command_args_slice(&svql_driver, &design, &module_name, openport));
        // Suppress direct stdout/stderr of the Yosys process; we'll capture it via wait_until_ready
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");

        // Wait for yosys to be ready
        let ready_marker = "SVQL DRIVER server listening on port";
        let timeout_secs: u64 = 10;

        wait_until_ready(&mut yosys_process, ready_marker, timeout_secs)?;

        let yosys_proc = YosysProc {
            child: yosys_process,
            port: openport,

            yosys,
            svql_driver,
            design,
            module_name,
        };
        Ok(yosys_proc)
    }
            

    pub fn driver(&self) -> Driver {
        let net_driver = Driver::new_net(format!("localhost:{}", self.port));
        net_driver
    }

    pub fn get_command(&self) -> String {
        get_command(&self.yosys, &self.svql_driver, &self.design, &self.module_name, self.port)
    }
}

fn get_command_args_slice(svql_driver: &Path, design: &Path, module_name: &str, port: u16) -> Vec<String> {
    vec![
        "-m".to_string(),
        format!("{}", svql_driver.display()),
        "-p".to_string(),
        format!("read_verilog {}", design.display()),
        "-p".to_string(),
        format!("hierarchy -top {}", module_name),
        "-p".to_string(),
        "proc".to_string(),
        "-p".to_string(),
        format!("svql_driver -net -port {}", port),
    ]
}

fn get_command(yosys: &Path, svql_driver: &Path, design: &Path, module_name: &str, port: u16) -> String {
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