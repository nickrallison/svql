use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use prjunnamed_netlist::Design;

use crate::DriverKey;

#[derive(Debug, Clone)]
pub struct Context {
    designs: HashMap<DriverKey, Arc<Design>>,
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
    pub fn with_design(mut self, key: DriverKey, design: Arc<Design>) -> Self {
        self.designs.insert(key.clone(), design);
        tracing::event!(tracing::Level::DEBUG, "Design added to context: {:?}", key);
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

    #[contracts::debug_ensures(self.get(&key).is_some())]
    pub(crate) fn insert(&mut self, key: DriverKey, design: Arc<Design>) {
        self.designs.insert(key.clone(), design);
        tracing::event!(tracing::Level::DEBUG, "Design inserted into context: {:?}", key);
    }

    #[contracts::debug_ensures(ret.len() >= self.len())]
    pub fn merge(mut self, other: Context) -> Self {
        for (key, design) in other.designs {
            self.designs.insert(key, design);
        }
        tracing::event!(tracing::Level::DEBUG, "Context merged, new size: {}", self.designs.len());
        self.clone()
    }

    /// Create a context from a single design
    #[contracts::debug_ensures(ret.len() == 1)]
    pub fn from_single(key: DriverKey, design: Arc<Design>) -> Self {
        let mut ctx = Self::new();
        ctx.designs.insert(key, design);
        tracing::event!(tracing::Level::DEBUG, "Single design context created");
        ctx
    }
}
