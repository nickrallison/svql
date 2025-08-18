use std::sync::Arc;

use prjunnamed_netlist::Design;

use crate::driver::DesignPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesignKey {
    pub path: DesignPath,
    pub top: String,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub cache: std::collections::HashMap<DesignKey, Arc<Design>>,
}

impl Cache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Insert a design into the cache for the given key
    pub fn insert(&mut self, key: DesignKey, design: Design) {
        self.cache.insert(key, Arc::new(design));
    }

    /// Get a design from the cache for the given key
    pub fn get(&self, key: &DesignKey) -> Option<Arc<Design>> {
        self.cache.get(key).cloned()
    }

    /// Check if the cache contains a design for the given key
    pub fn contains(&self, key: &DesignKey) -> bool {
        self.cache.contains_key(key)
    }

    /// Remove a design from the cache for the given key
    pub fn remove(&mut self, key: &DesignKey) -> Option<Arc<Design>> {
        self.cache.remove(key)
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
