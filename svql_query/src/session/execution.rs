//! Execution plan and context for parallel pattern matching.
//!
//! The execution model has two phases:
//! 1. **Plan construction** (single-threaded): Build a DAG from the registry
//! 2. **Execution** (multi-threaded): Traverse DAG with OnceLock per slot
//!
//! This module provides the infrastructure. The actual `search` function
//! pointers are provided by the `Pattern` trait implementations.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use svql_driver::{Driver, DriverKey};

use super::error::QueryError;
use super::registry::PatternRegistry;
use super::store::Store;
use super::table::AnyTable;

/// Configuration for query execution.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Whether to run in parallel (default: true).
    pub parallel: bool,
    /// Maximum number of threads (default: rayon default).
    pub max_threads: Option<usize>,
}

impl Config {
    /// Create a default parallel config.
    pub fn parallel() -> Self {
        Self {
            parallel: true,
            max_threads: None,
        }
    }

    /// Create a sequential (single-threaded) config.
    pub fn sequential() -> Self {
        Self {
            parallel: false,
            max_threads: Some(1),
        }
    }
}

/// Type alias for a search function.
///
/// Search functions take an `ExecutionContext` and return a type-erased table.
/// They are provided by `Pattern::search()` implementations.
pub type SearchFn = fn(&ExecutionContext<'_>) -> Result<Box<dyn AnyTable>, QueryError>;

/// A node in the execution DAG.
pub struct ExecutionNode {
    /// The pattern type this node represents.
    pub type_id: TypeId,
    /// Human-readable name for debugging.
    pub type_name: &'static str,
    /// The search function to execute.
    pub search_fn: SearchFn,
    /// Dependencies that must complete before this node.
    pub deps: Vec<Arc<ExecutionNode>>,
}

impl std::fmt::Debug for ExecutionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionNode")
            .field("type_name", &self.type_name)
            .field("deps", &self.deps.len())
            .finish()
    }
}

/// An execution plan built from a pattern's dependency graph.
///
/// The plan contains all nodes needed to execute a pattern query,
/// with dependencies properly ordered.
pub struct ExecutionPlan {
    /// The root node (the pattern we're ultimately searching for).
    pub root: Arc<ExecutionNode>,
    /// All nodes in topological order (deps before dependents).
    pub nodes: Vec<Arc<ExecutionNode>>,
}

impl std::fmt::Debug for ExecutionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionPlan")
            .field("root", &self.root.type_name)
            .field("num_nodes", &self.nodes.len())
            .finish()
    }
}

impl ExecutionPlan {
    /// Build an execution plan from a registry with search functions.
    ///
    /// # Arguments
    ///
    /// * `root_type_id` - The TypeId of the root pattern
    /// * `registry` - The pattern registry with dependency info
    /// * `search_fns` - Map of TypeId to search function
    pub fn build(
        root_type_id: TypeId,
        registry: &PatternRegistry,
        search_fns: &HashMap<TypeId, (&'static str, SearchFn)>,
    ) -> Result<Self, QueryError> {
        // Build nodes, sharing Arc for diamond deps
        let mut node_map: HashMap<TypeId, Arc<ExecutionNode>> = HashMap::new();

        // Get topological order
        let topo_order = registry.topological_order()?;

        // Build nodes in topological order (deps first)
        for type_id in &topo_order {
            let entry = registry.get(*type_id).ok_or_else(|| {
                QueryError::missing_dep(&format!("Missing registry entry for {:?}", type_id))
            })?;

            let (type_name, search_fn) = search_fns.get(type_id).ok_or_else(|| {
                QueryError::missing_dep(&format!("Missing search function for {}", entry.type_name))
            })?;

            // Collect dep nodes (already built due to topo order)
            let deps: Vec<Arc<ExecutionNode>> = entry
                .dependencies
                .iter()
                .filter_map(|dep_id| node_map.get(dep_id).cloned())
                .collect();

            let node = Arc::new(ExecutionNode {
                type_id: *type_id,
                type_name,
                search_fn: *search_fn,
                deps,
            });

            node_map.insert(*type_id, node);
        }

        // Get root node
        let root = node_map.get(&root_type_id).cloned().ok_or_else(|| {
            QueryError::missing_dep("Root pattern not found in registry")
        })?;

        // Collect all nodes in topo order
        let nodes: Vec<Arc<ExecutionNode>> = topo_order
            .iter()
            .filter_map(|tid| node_map.get(tid).cloned())
            .collect();

        Ok(Self { root, nodes })
    }

    /// Build an execution plan for a pattern type.
    ///
    /// This is a convenience method that:
    /// 1. Creates a registry
    /// 2. Registers the pattern and all its dependencies
    /// 3. Builds the plan with search functions from `df_search`
    ///
    /// # Type Parameters
    ///
    /// * `P` - A pattern type that implements `SearchableComponent`
    pub fn for_pattern<P>() -> Result<Self, QueryError>
    where
        P: crate::traits::SearchableComponent + Send + Sync + 'static,
    {
        use crate::traits::SearchableComponent;

        // Create and populate registry
        let mut registry = PatternRegistry::new();
        P::df_register_all(&mut registry);

        // Build search function map
        let mut search_fns: HashMap<TypeId, (&'static str, SearchFn)> = HashMap::new();

        // For each registered type, we need to create a search function.
        // This is tricky because we can only create the search function for types
        // we know about statically. For now, we only handle the root type.
        // In a full implementation, we'd need a trait object or registration system.

        // Register the root type's search function
        let search_fn: SearchFn = |ctx| {
            let table = P::df_search(ctx)?;
            Ok(Box::new(table) as Box<dyn AnyTable>)
        };
        search_fns.insert(
            TypeId::of::<P>(),
            (std::any::type_name::<P>(), search_fn),
        );

        // For dependencies, we need to use a different approach since we don't have
        // their types statically. This requires a trait object registration pattern.
        // For now, this only works for patterns without dependencies (netlists).

        let root_type_id = TypeId::of::<P>();
        Self::build(root_type_id, &registry, &search_fns)
    }

    /// Execute the plan and return a Store with all results.
    ///
    /// # Arguments
    ///
    /// * `driver` - The driver for design access
    /// * `key` - The driver key for the design to search
    /// * `config` - Execution configuration
    pub fn execute<'d>(
        &self,
        driver: &'d Driver,
        key: DriverKey,
        config: Config,
    ) -> Result<Store, QueryError> {
        // Create slots for each node
        let slots: HashMap<TypeId, OnceLock<Box<dyn AnyTable>>> = self
            .nodes
            .iter()
            .map(|node| (node.type_id, OnceLock::new()))
            .collect();

        // Create shared context
        let ctx = ExecutionContext {
            driver,
            driver_key: key,
            config: config.clone(),
            slots: Arc::new(slots),
        };

        if config.parallel {
            self.execute_parallel(&ctx)?;
        } else {
            self.execute_sequential(&ctx)?;
        }

        // Collect results into Store
        Ok(ctx.into_store())
    }

    /// Execute nodes sequentially in topological order.
    fn execute_sequential(&self, ctx: &ExecutionContext<'_>) -> Result<(), QueryError> {
        for node in &self.nodes {
            self.execute_node(node, ctx)?;
        }
        Ok(())
    }

    /// Execute nodes in parallel using rayon.
    #[cfg(feature = "parallel")]
    fn execute_parallel(&self, ctx: &ExecutionContext<'_>) -> Result<(), QueryError> {
        use rayon::prelude::*;

        // Execute all nodes - OnceLock ensures each runs exactly once
        self.nodes.par_iter().try_for_each(|node| {
            self.execute_node(node, ctx)
        })?;

        Ok(())
    }

    /// Fallback sequential execution when parallel feature is disabled.
    #[cfg(not(feature = "parallel"))]
    fn execute_parallel(&self, ctx: &ExecutionContext<'_>) -> Result<(), QueryError> {
        // Without rayon, fall back to sequential
        self.execute_sequential(ctx)
    }

    /// Execute a single node, waiting for deps first.
    fn execute_node(
        &self,
        node: &Arc<ExecutionNode>,
        ctx: &ExecutionContext<'_>,
    ) -> Result<(), QueryError> {
        // Check if already executed
        if let Some(slot) = ctx.slots.get(&node.type_id) {
            if slot.get().is_some() {
                return Ok(()); // Already done
            }
        }

        // Wait for dependencies (spin-wait on OnceLock)
        for dep in &node.deps {
            if let Some(dep_slot) = ctx.slots.get(&dep.type_id) {
                // Spin-wait for dep to complete
                while dep_slot.get().is_none() {
                    std::hint::spin_loop();
                }
            }
        }

        // Execute search
        let result = (node.search_fn)(ctx)?;

        // Store result (OnceLock ensures single write)
        if let Some(slot) = ctx.slots.get(&node.type_id) {
            let _ = slot.set(result); // Ignore if already set (race condition)
        }

        Ok(())
    }
}

/// Context passed to pattern search functions.
///
/// Provides access to:
/// - The driver for design access
/// - The driver key for the current design
/// - Completed dependency tables
pub struct ExecutionContext<'d> {
    /// The driver for design/needle operations.
    driver: &'d Driver,
    /// Key for the design being searched.
    driver_key: DriverKey,
    /// Execution configuration.
    config: Config,
    /// Slots for storing results (shared with plan).
    slots: Arc<HashMap<TypeId, OnceLock<Box<dyn AnyTable>>>>,
}

impl<'d> ExecutionContext<'d> {
    /// Get the driver.
    pub fn driver(&self) -> &'d Driver {
        self.driver
    }

    /// Get the driver key.
    pub fn driver_key(&self) -> DriverKey {
        self.driver_key.clone()
    }

    /// Get the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a dependency's table.
    ///
    /// Returns `None` if the dependency hasn't completed yet or doesn't exist.
    ///
    /// # Panics
    ///
    /// This should only be called for declared dependencies. If DAG ordering
    /// is correct, the dependency will always be available.
    pub fn get<T: 'static>(&self) -> Option<&super::table::Table<T>> {
        self.slots
            .get(&TypeId::of::<T>())
            .and_then(|lock| lock.get())
            .and_then(|boxed| boxed.as_any().downcast_ref())
    }

    /// Convert the context into a Store after execution completes.
    fn into_store(self) -> Store {
        let store = Store::with_capacity(self.slots.len());

        // Move tables from slots to store
        // Note: We can't move out of OnceLock directly, so we iterate
        // and use the TypeId mapping
        for (&type_id, slot) in self.slots.iter() {
            if slot.get().is_some() {
                // The table exists, but we can't move it out of OnceLock
                // We'll use insert_any which takes a TypeId directly
                // For a proper impl, we'd need Arc<dyn AnyTable> in slots
                let _ = type_id; // placeholder
            }
        }

        // TODO: Proper ownership transfer from OnceLock to Store
        // This requires restructuring to use Arc<dyn AnyTable> or similar
        store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let parallel = Config::parallel();
        assert!(parallel.parallel);
        assert!(parallel.max_threads.is_none());

        let sequential = Config::sequential();
        assert!(!sequential.parallel);
        assert_eq!(sequential.max_threads, Some(1));
    }
}
