use std::{sync::{Arc, RwLock}};

use prjunnamed_netlist::Design;

use crate::driver::DesignPath;


#[derive(Debug, Clone)]
pub struct Cache {
    pub cache: std::collections::HashMap<DesignPath, Arc<Design>>,
}

impl Cache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Insert a design into the cache for the given path
    pub fn insert(&mut self, path: DesignPath, design: Design) {
        self.cache.insert(path, Arc::new(design));
    }

    /// Get a design from the cache for the given path
    pub fn get(&self, path: &DesignPath) -> Option<Arc<Design>> {
        self.cache.get(path).cloned()
    }

    /// Check if the cache contains a design for the given path
    pub fn contains(&self, path: &DesignPath) -> bool {
        self.cache.contains_key(path)
    }

    /// Remove a design from the cache for the given path
    pub fn remove(&mut self, path: &DesignPath) -> Option<Arc<Design>> {
        self.cache.remove(path)
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

