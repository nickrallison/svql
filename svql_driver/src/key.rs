use std::{fmt, path::PathBuf};

use crate::util::DesignPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesignKey {
    path: DesignPath,
    top: String,
}

impl DesignKey {
    pub fn new<P: Into<PathBuf>>(path: P, top: String) -> Result<Self, String> {
        let design_path = DesignPath::new(path.into())
            .map_err(|e| format!("Failed to create design path: {}", e))?;
        Ok(Self {
            path: design_path,
            top,
        })
    }

    fn normalize<P: Into<PathBuf>>(self, root: P) -> Result<Self, Box<dyn std::error::Error>> {
        let root: PathBuf = root.into();
        let abs_root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()?.join(root)
        };

        let key_path: PathBuf = self.path.path().to_owned();
        if key_path.is_absolute() {
            return Ok(self);
        }

        let abs_key_path = abs_root.join(key_path);
        let canonicalized_path = abs_key_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        Ok(Self {
            path: DesignPath::new(canonicalized_path)?,
            top: self.top,
        })
    }
}

impl fmt::Display for DesignKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.path.path().display(), self.top)
    }
}
