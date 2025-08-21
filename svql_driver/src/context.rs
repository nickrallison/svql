// svql_driver/src/context.rs
use std::collections::HashMap;
use std::sync::Arc;

use crate::{manager::DesignHandle, manager::DesignManager};
use prjunnamed_netlist::Design;

#[derive(Debug, Clone)]
pub struct QueryContext {
    manager: DesignManager,
    designs: HashMap<String, DesignHandle>,
}

impl QueryContext {
    pub fn new(manager: DesignManager) -> Self {
        Self {
            manager,
            designs: HashMap::new(),
        }
    }

    /// Add a design to the context with a name
    pub fn add_design(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<std::path::Path>,
        module_name: &str,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let handle = self.manager.load_design(path, module_name)?;
        self.designs.insert(name.into(), handle);
        Ok(self)
    }

    /// Add multiple designs at once
    pub fn add_designs<I, P>(&mut self, designs: I) -> Result<&mut Self, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = (String, P, String)>,
        P: AsRef<std::path::Path>,
    {
        for (name, path, module_name) in designs {
            self.add_design(name, path, &module_name)?;
        }
        Ok(self)
    }

    /// Get a design handle by name
    pub fn get_design(&self, name: &str) -> Option<&DesignHandle> {
        self.designs.get(name)
    }

    /// Get design reference for subgraph matching
    pub fn get_design_ref(&self, name: &str) -> Option<Arc<Design>> {
        self.designs.get(name)?.get_design()
    }

    /// Create a context that's ready for querying
    pub fn build(self) -> ReadyContext {
        ReadyContext {
            designs: self.designs,
            manager: self.manager,
        }
    }
}

/// A context that's ready for actual querying operations
#[derive(Debug)]
pub struct ReadyContext {
    designs: HashMap<String, DesignHandle>,
    manager: DesignManager,
}

impl ReadyContext {
    pub fn get_design(&self, name: &str) -> Option<&DesignHandle> {
        self.designs.get(name)
    }

    pub fn get_design_ref(&self, name: &str) -> Option<Arc<Design>> {
        self.designs.get(name)?.get_design()
    }

    /// Execute a query function with access to all designs in context
    pub fn query<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&QueryScope) -> R,
    {
        let scope = QueryScope {
            designs: &self.designs,
            manager: &self.manager,
        };
        f(&scope)
    }
}

/// Scope provided to query functions with access to all context designs
pub struct QueryScope<'a> {
    designs: &'a HashMap<String, DesignHandle>,
    manager: &'a DesignManager,
}

impl<'a> QueryScope<'a> {
    pub fn get_design(&self, name: &str) -> Option<&DesignHandle> {
        self.designs.get(name)
    }

    pub fn get_design_ref(&self, name: &str) -> Option<Arc<Design>> {
        self.designs.get(name)?.get_design()
    }

    pub fn design_names(&self) -> impl Iterator<Item = &String> {
        self.designs.keys()
    }
}
