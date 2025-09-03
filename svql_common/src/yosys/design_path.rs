use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesignPath {
    Verilog(PathBuf),
    Rtlil(PathBuf),
    Json(PathBuf),
}

impl DesignPath {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("v") => Ok(DesignPath::Verilog(path)),
            Some("il") => Ok(DesignPath::Rtlil(path)),
            Some("json") => Ok(DesignPath::Json(path)),
            _ => Err(format!(
                "Unsupported design file extension: {:?}",
                path.extension()
            )),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            DesignPath::Verilog(p) | DesignPath::Rtlil(p) | DesignPath::Json(p) => p,
        }
    }

    pub fn read_command(&self) -> &'static str {
        match self {
            DesignPath::Verilog(_) => "read_verilog",
            DesignPath::Rtlil(_) => "read_rtlil",
            DesignPath::Json(_) => "read_json",
        }
    }
}
