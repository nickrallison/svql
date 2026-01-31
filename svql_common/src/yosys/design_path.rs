//! Handling of design file paths and types.

use std::path::{Path, PathBuf};

/// Represents a path to a design file, categorized by its type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesignPath {
    /// A Verilog file (.v).
    Verilog(PathBuf),
    /// An RTLIL file (.il).
    Rtlil(PathBuf),
    /// A JSON file (.json).
    Json(PathBuf),
}

impl DesignPath {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("v") => return Ok(Self::Verilog(path)),
            Some("il") => return Ok(Self::Rtlil(path)),
            Some("json") => return Ok(Self::Json(path)),
            _ => Err(format!(
                "Unsupported design file extension: {:?}",
                path.extension()
            )),
        }
    }

    #[must_use] 
    pub fn path(&self) -> &Path {
        match self {
            Self::Verilog(p) | Self::Rtlil(p) | Self::Json(p) => p,
        }
    }

    #[must_use] 
    pub const fn read_command(&self) -> &'static str {
        match self {
            Self::Verilog(_) => "read_verilog -sv",
            Self::Rtlil(_) => "read_rtlil",
            Self::Json(_) => "read_json",
        }
    }
}
