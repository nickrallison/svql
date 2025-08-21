use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use prjunnamed_netlist::Design;

use crate::DriverKey;

#[derive(Debug, Clone, Default)]
pub struct Context {
    designs: HashMap<DriverKey, Arc<Design>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            designs: HashMap::new(),
        }
    }

    pub fn with_design(mut self, key: DriverKey, design: Arc<Design>) -> Self {
        self.designs.insert(key, design);
        self
    }

    pub fn get(&self, key: &DriverKey) -> Option<&Arc<Design>> {
        self.designs.get(key)
    }

    pub fn get_by_path_and_module<P: AsRef<Path>>(
        &self,
        path: P,
        module_name: &str,
    ) -> Option<&Arc<Design>> {
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

    pub fn iter(&self) -> impl Iterator<Item = (&DriverKey, &Arc<Design>)> {
        self.designs.iter()
    }

    pub(crate) fn insert(&mut self, key: DriverKey, design: Arc<Design>) {
        self.designs.insert(key, design);
    }

    /// Merge another context into this one, returning a new context
    pub fn merge(mut self, other: Context) -> Self {
        for (key, design) in other.designs {
            self.designs.insert(key, design);
        }
        self
    }

    /// Create a context from a single design
    pub fn from_single(key: DriverKey, design: Arc<Design>) -> Self {
        let mut ctx = Self::new();
        ctx.designs.insert(key, design);
        ctx
    }
}
