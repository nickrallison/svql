use std::{collections::HashMap, sync::Arc};

use prjunnamed_netlist::Design;

pub struct Context {
    map: HashMap<String, Arc<Design>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: String, design: Arc<Design>) {
        self.map.insert(path, design);
    }

    pub fn get(&self, path: &str) -> Option<&Arc<Design>> {
        self.map.get(path)
    }

    pub fn extend(&mut self, other: Context) {
        self.map.extend(other.map);
    }
}
