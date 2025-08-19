use log::{error, trace};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::Stdio,
};

use which::which;

enum YosysOutput {
    RTLIL(PathBuf),
    JSON(PathBuf),
}

impl YosysOutput {
    fn path(&self) -> &PathBuf {
        match self {
            YosysOutput::RTLIL(path) => path,
            YosysOutput::JSON(path) => path,
        }
    }
}

/// Walk a directory tree and return every file whose extension matches `ext`.
fn collect_files_with_ext(root: &Path, ext: &str) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut dirs = vec![root.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().and_then(OsStr::to_str) == Some(ext) {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Build the argument list for a Yosys invocation that reads a Verilog file and
/// writes an RTLIL file.
fn get_args(verilog: &Path, module_name: &str, file_out: YosysOutput) -> Vec<String> {
    let write_cmd = match file_out {
        YosysOutput::RTLIL(path) => format!("write_rtlil {}", path.display()),
        YosysOutput::JSON(path) => format!("write_json {}", path.display()),
    };

    vec![
        "-p".to_string(),
        format!("read_verilog {}", verilog.display()),
        "-p".to_string(),
        format!("hierarchy -top {}", module_name),
        "-p".to_string(),
        "proc; flatten; opt_clean".to_string(),
        "-p".to_string(),
        write_cmd,
    ]
}

/// Run Yosys on a single Verilog file and produce the corresponding RTLIL file.
fn run_yosys(
    yosys: &Path,
    verilog: &Path,
    module_name: &str,
    file_out: YosysOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!(
        "Running Yosys: {:?} on {:?} → {:?}",
        yosys,
        verilog,
        file_out.path()
    );

    let mut cmd = std::process::Command::new(yosys);
    cmd.args(get_args(verilog, module_name, file_out));
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Yosys: {}", e))?;
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for Yosys: {}", e))?;

    if !status.success() {
        let mut stderr = child
            .stderr
            .take()
            .ok_or("Failed to capture Yosys stderr")?;
        let mut buf = Vec::new();
        use std::io::Read;
        stderr
            .read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read Yosys stderr: {}", e))?;
        let msg = String::from_utf8_lossy(&buf);
        error!(
            "Yosys failed (status: {:?})\nStderr: {}",
            status.code(),
            msg
        );
        return Err(format!(
            "Yosys failed (status: {:?})\nStderr: {}",
            status.code(),
            msg
        )
        .into());
    }

    Ok(())
}

/// Main entry point for the helper binary.
///
/// Usage:
/// ```text
/// cargo run -p svql_driver --bin util_fill_examples <examples_root>
/// ```
///
/// `<examples_root>` must contain a `verilog/` directory with `.v` files.  For each
/// file the tool creates (if necessary) a sibling `rtlil/` directory and writes
/// the generated RTLIL file there, preserving the original file name but with an
/// `.il` extension.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise logger (trace level gives the most detail while debugging).
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // -------------------------------------------------------------------------
    // Argument handling
    // -------------------------------------------------------------------------
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <examples_root>", args[0]);
        std::process::exit(1);
    }
    let examples_root = PathBuf::from(&args[1]);

    // Verify the expected sub‑directories exist.
    let verilog_dir = examples_root.join("verilog");
    let rtlil_dir = examples_root.join("rtlil");
    let json_dir = examples_root.join("json");

    if !verilog_dir.is_dir() {
        return Err(format!(
            "Verilog source directory not found: {}",
            verilog_dir.display()
        )
        .into());
    }
    // Create the rtlil output directory if it does not already exist.
    fs::create_dir_all(&rtlil_dir).map_err(|e| {
        format!(
            "Failed to create rtlil output directory {}: {}",
            rtlil_dir.display(),
            e
        )
    })?;

    fs::create_dir_all(&json_dir).map_err(|e| {
        format!(
            "Failed to create json output directory {}: {}",
            json_dir.display(),
            e
        )
    })?;

    // -------------------------------------------------------------------------
    // Locate all Verilog files
    // -------------------------------------------------------------------------
    let verilog_files = collect_files_with_ext(&verilog_dir, "v")?;
    if verilog_files.is_empty() {
        println!("No Verilog files found under {}", verilog_dir.display());
        return Ok(());
    }

    // -------------------------------------------------------------------------
    // Locate Yosys binary once (fails early if missing)
    // -------------------------------------------------------------------------
    let yosys_path = which("yosys").map_err(|e| format!("Failed to locate Yosys binary: {}", e))?;

    // -------------------------------------------------------------------------
    // Process each file
    // -------------------------------------------------------------------------
    for verilog_path in verilog_files {
        // Derive module name from the file stem.
        let module_name = verilog_path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                format!(
                    "Failed to extract module name from path {}",
                    verilog_path.display()
                )
            })?;

        // Destination RTLIL path mirrors the source name but lives under rtlil/.
        let rtlil_path = rtlil_dir.join(
            verilog_path
                .file_name()
                .ok_or("Verilog path has no file name")?
                .to_string_lossy()
                .replace(".v", ".il"),
        );

        let json_path = json_dir.join(
            verilog_path
                .file_name()
                .ok_or("Verilog path has no file name")?
                .to_string_lossy()
                .replace(".v", ".json"),
        );

        // Skip if the RTLIL file already exists (optional – remove this block to
        // force regeneration).
        if rtlil_path.exists() {
            trace!(
                "Skipping {} – RTLIL already present at {}",
                verilog_path.display(),
                rtlil_path.display()
            );
        } else {
            let rtlil_out = YosysOutput::RTLIL(rtlil_path.clone());
            match run_yosys(&yosys_path, &verilog_path, module_name, rtlil_out) {
                Ok(_) => {
                    println!(
                        "Generated RTLIL: {} → {}",
                        verilog_path.display(),
                        rtlil_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", verilog_path.display(), e);
                }
            }
        }

        if json_path.exists() {
            trace!(
                "Skipping {} – JSON already present at {}",
                verilog_path.display(),
                json_path.display()
            );
        } else {
            let json_out = YosysOutput::JSON(json_path.clone());
            match run_yosys(&yosys_path, &verilog_path, module_name, json_out) {
                Ok(_) => {
                    println!(
                        "Generated JSON: {} → {}",
                        verilog_path.display(),
                        json_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", verilog_path.display(), e);
                }
            }
        }
    }

    Ok(())
}
