//! Handling of design file paths and types.

use contracts::*;
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
    /// Categorizes a filesystem path based on its extension.
    ///
    /// # Errors
    ///
    /// Returns an error string if the file extension is missing or not
    /// recognized as a supported design format (.v, .il, .json).
    #[ensures(ret.as_ref().map(|dp| dp.path() == path).unwrap_or(true))]
    pub fn new(path: PathBuf) -> Result<Self, String> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("v") => Ok(Self::Verilog(path.clone())),
            Some("il") => Ok(Self::Rtlil(path.clone())),
            Some("json") => Ok(Self::Json(path.clone())),
            _ => Err(format!(
                "Unsupported design file extension: {:?}",
                path.extension()
            )),
        }
    }

    /// Returns a reference to the underlying path.
    #[must_use]
    #[ensures(ret == match self { Self::Verilog(p) | Self::Rtlil(p) | Self::Json(p) => p })]
    pub fn path(&self) -> &Path {
        match self {
            Self::Verilog(p) | Self::Rtlil(p) | Self::Json(p) => p,
        }
    }

    /// Returns the Yosys command string used to read this file type.
    #[must_use]
    #[ensures(!ret.is_empty())]
    pub const fn read_command(&self) -> &'static str {
        match self {
            Self::Verilog(_) => "read_verilog -sv",
            Self::Rtlil(_) => "read_rtlil",
            Self::Json(_) => "read_json",
        }
    }
}
