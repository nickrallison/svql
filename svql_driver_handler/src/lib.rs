use std::{path::{Path, PathBuf}, process::Child};
use driver::Driver;

pub mod driver;

#[derive(Debug)]
pub struct YosysProc {
    child: Child,
    port: u16,

    // ###
    yosys: PathBuf,
    design: PathBuf,
    svql_driver: PathBuf,
}

impl YosysProc {
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

    pub fn new_yosys(yosys: PathBuf, svql_driver: PathBuf, design: PathBuf, module_name: String) -> Result<Self, String> {

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

        let mut yosys_process = std::process::Command::new("yosys")
            .arg("-p")
            .arg(format!("\"read_verilog {}\"", design.display()))
            .arg("-p")
            .arg(format!("\"hierarchy -top {}\"", module_name))
            .arg("-p")
            .arg("proc")
            .arg("-p")
            .arg(format!("\"svql_driver -net -port {}\"", openport))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start yosys process");

        // Wait for yosys to be ready
        let ready_marker = "SVQL DRIVER server listening on port";
        let timeout_secs: u64 = 10;

        wait_until_ready(&mut yosys_process, ready_marker, timeout_secs)?;

        let yosys_proc = YosysProc {
            child: yosys_process,
            port: openport,

            yosys,
            design,
            svql_driver,
        };
        Ok(yosys_proc)
    }

    pub fn driver(&self) -> Driver {
        let net_driver = Driver::new_net(format!("localhost:{}", self.port));
        net_driver
    }
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

    let mut disconnected = false;
    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if recent.len() == recent.capacity() {
                    recent.pop_front();
                }
                recent.push_back(line.clone());
                if line.contains(&ready_marker) {
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
                disconnected = true;
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    if !ready {
        return Err(format!(
            "Timed out waiting for yosys to be ready. Expected line containing: '{}'.\nRecent output:\n{}",
            ready_marker,
            recent.into_iter().collect::<Vec<_>>().join("\n")
        ));
    }

    Ok(())
}