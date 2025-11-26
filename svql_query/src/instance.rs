//! Hierarchical instance path representation.

use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instance {
    pub segments: Vec<Arc<str>>,
}

impl Instance {
    pub fn root(name: String) -> Self {
        // <- FIXED: String, not Option
        Self {
            segments: vec![Arc::from(name)],
        }
    }

    pub fn child(&self, name: &str) -> Instance {
        let mut segments = self.segments.clone();
        segments.push(Arc::from(name));
        Instance { segments }
    }

    pub fn starts_with(&self, prefix: &Instance) -> bool {
        if prefix.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..prefix.segments.len()] == prefix.segments[..]
    }

    pub fn relative(&self, prefix: &Self) -> &[Arc<str>] {
        if !self.starts_with(prefix) {
            panic!(
                "Instance {:?} does not start with prefix {:?}",
                self, prefix
            );
        }
        &self.segments[prefix.segments.len()..]
    }

    pub fn inst_path(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    // Helper for legacy code that might use height/get_item
    pub fn height(&self) -> usize {
        self.segments.len()
    }

    pub fn get_item(&self, idx: usize) -> Option<Arc<str>> {
        self.segments.get(idx).cloned()
    }
}
