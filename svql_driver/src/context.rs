//! Context management for SVQL designs.
//!
//! This module defines the `Context` type, which holds a collection of loaded designs
//! keyed by their `DriverKey`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::DriverKey;
use crate::design_container::DesignContainer;

/// A collection of loaded designs available for querying.
#[derive(Debug, Clone)]
pub struct Context {
    designs: HashMap<DriverKey, Arc<DesignContainer>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            designs: HashMap::new(),
        }
    }

    #[contracts::debug_ensures(ret.get(&key).is_some())]
    pub fn with_design(mut self, key: DriverKey, design: Arc<DesignContainer>) -> Self {
        self.designs.insert(key.clone(), design);
        tracing::event!(tracing::Level::DEBUG, "Design added to context: {:?}", key);
        self
    }

    pub fn get(&self, key: &DriverKey) -> Option<&Arc<DesignContainer>> {
        self.designs.get(key)
    }

    pub fn get_by_path_and_module<P: AsRef<Path>>(
        &self,
        path: P,
        module_name: &str,
    ) -> Option<&Arc<DesignContainer>> {
        let key = DriverKey::new(path, module_name.to_string());
        self.designs.get(&key)
    }

    pub fn contains(&self, key: &DriverKey) -> bool {
        self.designs.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.designs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.designs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DriverKey, &Arc<DesignContainer>)> {
        self.designs.iter()
    }

    #[contracts::debug_ensures(self.get(&key).is_some())]
    pub(crate) fn insert(&mut self, key: DriverKey, design: Arc<DesignContainer>) {
        self.designs.insert(key.clone(), design);
        tracing::event!(
            tracing::Level::DEBUG,
            "Design inserted into context: {:?}",
            key
        );
    }

    pub fn merge(mut self, other: Context) -> Self {
        for (key, design) in other.designs {
            self.designs.insert(key, design);
        }
        tracing::event!(
            tracing::Level::DEBUG,
            "Context merged, new size: {}",
            self.designs.len()
        );
        self
    }

    #[contracts::debug_ensures(ret.len() == 1)]
    pub fn from_single(key: DriverKey, design: Arc<DesignContainer>) -> Self {
        let mut ctx = Self::new();
        ctx.designs.insert(key, design); // Internal, ok
        tracing::event!(tracing::Level::DEBUG, "Single design context created");
        ctx
    }
}
