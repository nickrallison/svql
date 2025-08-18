// This integration test demonstrates consumer usage with the `Finder` API.
// It requires `yosys` in PATH to convert the example Verilog into JSON.
//
// Enable with: `cargo test --features yosys-tests --test usage_yosys`

#![cfg(feature = "yosys-tests")]

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use prjunnamed_netlist::Design;
use svql_subgraph::Finder;

#[test]
fn consumer_can_find_sdffe_in_seq_double() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    let pat = load_design_from(repo_root.join("examples/patterns/basic/ff/sdffe.v"))
        .expect("failed to load pattern design");
    let hay = load_design_from(repo_root.join("examples/patterns/basic/ff/seq_double_sdffe.v"))
        .expect("failed to load haystack design");

    let results = Finder::new(&pat, &hay).find_all();
    assert_eq!(results.len(), 2, "expected exactly two matches");
}

// Simple Yosys-based loader, local to this test (consumer-side).
fn load_design_from(path: PathBuf) -> Result<Design, Box<dyn std::error::Error>> {
    let json_temp_file = tempfile::Builder::new()
        .prefix("svql_prjunnamed_")
        .suffix(".json")
        .rand_bytes(4)
        .tempfile()?;

    let yosys = which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;
    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            format!(
                "Failed to extract module name from path: {}",
                path.display()
            )
        })?
        .to_string();

    let read_cmd = match path.extension().and_then(|s| s.to_str()) {
        Some("v") => "read_verilog",
        Some("il") => "read_rtlil",
        Some("json") => "read_json",
        other => return Err(format!("Unsupported file extension: {:?}", other).into()),
    };

    let mut cmd = std::process::Command::new(yosys);
    cmd.args([
        "-p",
        &format!("{} {}", read_cmd, path.display()),
        "-p",
        &format!("hierarchy -top {}", module_name),
        "-p",
        "proc; flatten; opt_clean",
        "-p",
        &format!("write_json {}", json_temp_file.path().display()),
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::null());

    let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");
    let status = yosys_process
        .wait()
        .expect("Failed to wait for yosys process");
    if !status.success() {
        let mut stderr = yosys_process
            .stderr
            .take()
            .expect("Failed to capture stderr");
        let mut buf = Vec::new();
        use std::io::Read;
        stderr.read_to_end(&mut buf).expect("read stderr");
        let s = String::from_utf8_lossy(&buf);
        return Err(format!("Yosys failed with status {:?}\nstderr:\n{}", status, s).into());
    }

    let designs = prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;
    assert_eq!(designs.len(), 1);
    let design = designs.into_values().next().unwrap();
    Ok(design)
}
