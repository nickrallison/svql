// svql_pat/src/lib.rs

//! # SVQL Pattern Extraction Library
//!
//! This library provides functionality to extract interface patterns from Verilog files
//! using the yosys synthesis tool with the `svql_pat_lib` plugin. It can analyze Verilog
//! modules and extract information about their input, output, and inout ports.
//!
//! ## Features
//!
//! - Extract module interface patterns from Verilog files
//! - Comprehensive error handling for common failure modes
//! - Support for custom yosys and plugin paths
//! - JSON serialization of extracted patterns
//! - Detailed error reporting with helpful suggestions
//!
//! ## Usage
//!
//! ### Basic usage with default paths:
//! ```no_run
//! # use svql_pat::extract_pattern_default;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let pattern = extract_pattern_default("path/to/file.v", "module_name")?;
//! println!("Input ports: {:?}", pattern.in_ports);
//! println!("Output ports: {:?}", pattern.out_ports);
//! # Ok(())
//! # }
//! ```
//!
//! ### Advanced usage with custom paths:
//! ```no_run
//! # use svql_pat::extract_pattern;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let pattern = extract_pattern(
//!     "path/to/file.v",
//!     "module_name",
//!     Some("/path/to/yosys"),
//!     Some("/path/to/libsvql_pat_lib.so")
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The library provides detailed error types that help diagnose issues:
//! - `FileNotFound`: Verilog file doesn't exist
//! - `ModuleNotFound`: Module not found in the file
//! - `SyntaxError`: Verilog syntax errors
//! - `YosysExecutionError`: Problems running yosys
//! - `ParseError`: Issues parsing yosys output
//!
//! ## Requirements
//!
//! - Yosys synthesis tool must be installed and accessible
//! - The `svql_pat_lib.so` plugin must be built and available
//! - Input Verilog files must be syntactically correct

use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use svql_common::pattern::ffi::Pattern;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvqlPatError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Module '{module}' not found in file '{file}'")]
    ModuleNotFound { module: String, file: PathBuf },

    #[error("Verilog syntax error in file '{file}': {details}")]
    SyntaxError { file: PathBuf, details: String },

    #[error("Failed to execute yosys: {details}")]
    YosysExecutionError { details: String },

    #[error("Failed to parse yosys output: {details}")]
    ParseError { details: String },

    #[error("JSON parsing error: {details}")]
    JsonError { details: String },

    #[error("Pattern creation failed: {details}")]
    PatternCreationError { details: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SvqlPatError>;

/// Extract pattern information from a Verilog file using yosys
///
/// # Arguments
/// * `verilog_file` - Path to the Verilog file
/// * `module_name` - Name of the module to extract pattern from
/// * `yosys_bin_path` - Optional path to yosys binary (defaults to finding it in the workspace)
/// * `plugin_lib_path` - Optional path to the svql_pat_lib plugin (defaults to finding it in build dir)
///
/// # Returns
/// A Result containing the Pattern or an error
pub fn extract_pattern<P: AsRef<Path>>(
    verilog_file: P,
    module_name: &str,
    yosys_bin_path: Option<P>,
    plugin_lib_path: Option<P>,
) -> Result<Pattern> {
    let verilog_path = verilog_file.as_ref();

    // Check if file exists
    if !verilog_path.exists() {
        return Err(SvqlPatError::FileNotFound {
            path: verilog_path.to_path_buf(),
        });
    }

    // Determine paths to yosys and plugin
    let yosys_bin = if let Some(path) = yosys_bin_path {
        path.as_ref().to_path_buf()
    } else {
        find_yosys_binary()?
    };

    let plugin_lib = if let Some(path) = plugin_lib_path {
        path.as_ref().to_path_buf()
    } else {
        find_plugin_library()?
    };

    // Build yosys command
    let mut cmd = Command::new(&yosys_bin);
    cmd.arg("-m")
        .arg(&plugin_lib)
        .arg(verilog_path)
        .arg("-p")
        .arg(format!(
            "hierarchy -top {}; svql_pat -module {} -pattern_file {}",
            module_name,
            module_name,
            verilog_path.display()
        ));

    // Execute yosys
    let output = cmd
        .output()
        .map_err(|e| SvqlPatError::YosysExecutionError {
            details: format!("Failed to run yosys: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for errors in the output
    if !output.status.success() {
        // Check if it's a module not found error from yosys hierarchy command
        if stderr.contains("Module") && stderr.contains("not found") {
            return Err(SvqlPatError::ModuleNotFound {
                module: module_name.to_string(),
                file: verilog_path.to_path_buf(),
            });
        }

        return Err(SvqlPatError::YosysExecutionError {
            details: format!(
                "Yosys exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            ),
        });
    }

    // Parse the output for specific errors
    parse_yosys_output(&stdout, &stderr, verilog_path, module_name)
}

/// Parse yosys output to extract pattern or detect errors
fn parse_yosys_output(
    stdout: &str,
    stderr: &str,
    verilog_file: &Path,
    module_name: &str,
) -> Result<Pattern> {
    // Check for SVQL_PAT_ERROR markers
    if let Some(error_line) = stdout.lines().find(|line| line.contains("SVQL_PAT_ERROR:")) {
        if error_line.contains("Module") && error_line.contains("not found") {
            return Err(SvqlPatError::ModuleNotFound {
                module: module_name.to_string(),
                file: verilog_file.to_path_buf(),
            });
        }
        if error_line.contains("No module name specified") {
            return Err(SvqlPatError::PatternCreationError {
                details: "No module name specified".to_string(),
            });
        }
        if error_line.contains("Failed to create pattern") {
            return Err(SvqlPatError::PatternCreationError {
                details: "Failed to create pattern".to_string(),
            });
        }
        if error_line.contains("Failed to serialize pattern") {
            return Err(SvqlPatError::PatternCreationError {
                details: "Failed to serialize pattern to JSON".to_string(),
            });
        }
    }

    // Check for Verilog syntax errors in stderr
    if stderr.contains("syntax error") || stderr.contains("Parse error") {
        return Err(SvqlPatError::SyntaxError {
            file: verilog_file.to_path_buf(),
            details: extract_syntax_error_details(stderr),
        });
    }

    // Look for JSON pattern between markers
    let json_regex =
        Regex::new(r"(?s)SVQL_PAT_JSON_BEGIN\n(.*?)\nSVQL_PAT_JSON_END").map_err(|e| {
            SvqlPatError::ParseError {
                details: format!("Failed to create regex: {e}"),
            }
        })?;

    if let Some(captures) = json_regex.captures(stdout) {
        let json_str = captures.get(1).unwrap().as_str();

        // Parse JSON to Pattern
        serde_json::from_str::<Pattern>(json_str).map_err(|e| SvqlPatError::JsonError {
            details: format!("Failed to parse JSON: {e}"),
        })
    } else {
        Err(SvqlPatError::ParseError {
            details: "Could not find pattern JSON in yosys output".to_string(),
        })
    }
}

/// Extract details from syntax error messages
fn extract_syntax_error_details(stderr: &str) -> String {
    // Look for syntax error lines and extract relevant information
    let syntax_lines: Vec<&str> = stderr
        .lines()
        .filter(|line| {
            line.contains("syntax error") || line.contains("Parse error") || line.contains("ERROR:")
        })
        .take(3) // Take first few error lines
        .collect();

    if syntax_lines.is_empty() {
        "Unknown syntax error".to_string()
    } else {
        syntax_lines.join("\n")
    }
}

/// Find yosys binary in the workspace
fn find_yosys_binary() -> Result<PathBuf> {
    // Try common locations relative to the workspace root
    let possible_paths = [
        "yosys/yosys",
        "./yosys/yosys",
        "../yosys/yosys",
        "build/yosys/yosys",
    ];

    for path in &possible_paths {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Try system PATH
    if let Ok(output) = Command::new("which").arg("yosys").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path_str = path_str.trim();
            return Ok(PathBuf::from(path_str));
        }
    }

    Err(SvqlPatError::YosysExecutionError {
        details: "Could not find yosys binary".to_string(),
    })
}

/// Find the svql_pat_lib plugin library
fn find_plugin_library() -> Result<PathBuf> {
    let possible_paths = [
        "build/svql_pat_lib/libsvql_pat_lib.so",
        "./build/svql_pat_lib/libsvql_pat_lib.so",
        "../build/svql_pat_lib/libsvql_pat_lib.so",
        "svql_pat_lib/libsvql_pat_lib.so",
    ];

    for path in &possible_paths {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(SvqlPatError::YosysExecutionError {
        details: "Could not find svql_pat_lib plugin library".to_string(),
    })
}

/// Convenience function that uses default paths
pub fn extract_pattern_default<P: AsRef<Path>>(
    verilog_file: P,
    module_name: &str,
) -> Result<Pattern> {
    let verilog_path = verilog_file.as_ref();

    // Check if file exists
    if !verilog_path.exists() {
        return Err(SvqlPatError::FileNotFound {
            path: verilog_path.to_path_buf(),
        });
    }

    // Determine paths to yosys and plugin
    let yosys_bin = find_yosys_binary()?;
    let plugin_lib = find_plugin_library()?;

    // Build yosys command
    let mut cmd = Command::new(&yosys_bin);
    cmd.arg("-m")
        .arg(&plugin_lib)
        .arg(verilog_path)
        .arg("-p")
        .arg(format!(
            "hierarchy -top {}; svql_pat -module {} -pattern_file {}",
            module_name,
            module_name,
            verilog_path.display()
        ));

    // Execute yosys
    let output = cmd
        .output()
        .map_err(|e| SvqlPatError::YosysExecutionError {
            details: format!("Failed to run yosys: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for errors in the output
    if !output.status.success() {
        // Check if it's a module not found error from yosys hierarchy command
        if stderr.contains("Module") && stderr.contains("not found") {
            return Err(SvqlPatError::ModuleNotFound {
                module: module_name.to_string(),
                file: verilog_path.to_path_buf(),
            });
        }

        return Err(SvqlPatError::YosysExecutionError {
            details: format!(
                "Yosys exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            ),
        });
    }

    // Parse the output for specific errors
    parse_yosys_output(&stdout, &stderr, verilog_path, module_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        // Test that our error types can be created
        let _error = SvqlPatError::FileNotFound {
            path: PathBuf::from("test.v"),
        };
    }

    #[test]
    fn test_parse_syntax_error() {
        let stderr = "ERROR: Syntax error in line 5\nParse error: unexpected token";
        let details = extract_syntax_error_details(stderr);
        assert!(details.contains("Syntax error"));
        assert!(details.contains("Parse error"));
    }
}
