//! Hierarchical instance path representation.
//!
//! Provides the `Instance` type used to track the location of components
//! within a nested query or design hierarchy.

use std::sync::Arc;

/// Represents a hierarchical path in a design or query.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instance {
    /// Ordered segments of the path (e.g., ["top", "sub", "gate"]).
    pub segments: Vec<Arc<str>>,
}

impl Instance {
    /// Creates a root instance with a single name segment.
    pub fn root(name: String) -> Self {
        Self {
            segments: vec![Arc::from(name)],
        }
    }

    /// Creates a new instance representing a child of the current path.
    pub fn child(&self, name: &str) -> Instance {
        let mut segments = self.segments.clone();
        segments.push(Arc::from(name));
        Instance { segments }
    }

    /// Checks if this instance path starts with the provided prefix.
    pub fn starts_with(&self, prefix: &Instance) -> bool {
        if prefix.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..prefix.segments.len()] == prefix.segments[..]
    }

    /// Returns the relative path segments after the provided prefix.
    ///
    /// # Panics
    /// Panics if the instance does not start with the prefix.
    pub fn relative(&self, prefix: &Self) -> &[Arc<str>] {
        if !self.starts_with(prefix) {
            panic!(
                "Instance {:?} does not start with prefix {:?}",
                self, prefix
            );
        }
        &self.segments[prefix.segments.len()..]
    }

    /// Returns a dot-separated string representation of the path.
    pub fn inst_path(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Returns the number of segments in the path.
    pub fn height(&self) -> usize {
        self.segments.len()
    }

    /// Retrieves a specific segment of the path by index.
    pub fn get_item(&self, idx: usize) -> Option<Arc<str>> {
        self.segments.get(idx).cloned()
    }
}
