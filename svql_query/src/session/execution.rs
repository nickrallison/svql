//! Execution plan and context for parallel pattern matching.
//!
//! The execution model has two phases:
//! 1. **Plan construction** (single-threaded): Build a DAG from the registry
//! 2. **Execution** (multi-threaded): Traverse DAG with OnceLock per slot
//!
//! This module provides the infrastructure. The actual `search` function
//! pointers are provided by the `Pattern` trait implementations.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use svql_driver::{Driver, DriverKey};

use crate::traits::Pattern;

use super::error::QueryError;
use super::registry::PatternRegistry;
use super::store::Store;
use super::table::AnyTable;

/// Type alias for a search function.
///
/// Search functions take an `ExecutionContext` and return a type-erased table.
/// They are provided by `Pattern::search()` implementations.
pub type SearchFn = fn(&ExecutionContext<'_>) -> Result<Box<dyn AnyTable>, QueryError>;

/// Slot for storing a table during execution.
///
/// Uses `OnceLock<Arc<dyn AnyTable>>` so tables can be shared during execution
/// and then cloned into the final Store.
type TableSlot = OnceLock<Arc<dyn AnyTable>>;

/// A node in the execution DAG.
pub struct ExecutionNode {
    /// The pattern type this node represents.
    pub type_id: TypeId,
    /// Human-readable name for debugging.
    pub type_name: &'static str,
    /// The search function to execute.
    pub search_fn: SearchFn,
    /// Atomic flag to ensure single execution in parallel mode.
    pub cas_runner: AtomicBool,
    /// Dependencies that must complete before this node.
    pub deps: Vec<Arc<ExecutionNode>>,
}

impl ExecutionNode {
    /// Check if this node has already been executed.
    pub fn is_executed(&self) -> bool {
        self.cas_runner.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_done(&self, ctx: &ExecutionContext<'_>) -> bool {
        if let Some(slot) = ctx.slots.get(&self.type_id)
            && slot.get().is_some()
        {
            return true;
        }
        false
    }

    pub fn try_execute(&self) -> bool {
        let cas_result = self.cas_runner.compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        );
        cas_result.is_ok()
    }
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
                QueryError::missing_dep(format!("Missing registry entry for {:?}", type_id))
            })?;

            let (type_name, search_fn) = search_fns.get(type_id).ok_or_else(|| {
                QueryError::missing_dep(format!("Missing search function for {}", entry.type_name))
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
                cas_runner: AtomicBool::new(false),
                deps,
            });

            node_map.insert(*type_id, node);
        }

        // Get root node
        let root = node_map
            .get(&root_type_id)
            .cloned()
            .ok_or_else(|| QueryError::missing_dep("Root pattern not found in registry"))?;

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
    /// 1. Creates a search registry
    /// 2. Registers the pattern and all its dependencies with their search functions
    /// 3. Builds the plan
    ///
    /// This works for all pattern types (netlist, composite, variant).
    ///
    /// # Type Parameters
    ///
    /// * `P` - A pattern type that implements `SearchableComponent`
    pub fn for_pattern<P>() -> Result<Self, QueryError>
    where
        P: Pattern + Send + Sync + 'static,
    {
        use super::registry::SearchRegistry;

        // Create and populate the combined registry with search functions
        let mut registry = SearchRegistry::new();
        P::df_register_search(&mut registry);

        // Build the plan using the combined registry
        Self::from_search_registry(TypeId::of::<P>(), registry)
    }

    /// Build an execution plan from a SearchRegistry.
    ///
    /// The SearchRegistry contains both pattern metadata and search functions,
    /// collected via `df_register_search` calls.
    pub fn from_search_registry(
        root_type_id: TypeId,
        registry: super::registry::SearchRegistry,
    ) -> Result<Self, QueryError> {
        // Build nodes, sharing Arc for diamond deps
        let mut node_map: HashMap<TypeId, Arc<ExecutionNode>> = HashMap::new();

        // Get topological order
        let topo_order = registry.topological_order()?;

        // Build nodes in topological order (deps first)
        for type_id in &topo_order {
            let entry = registry.get(*type_id).ok_or_else(|| {
                QueryError::missing_dep(format!("Missing registry entry for {:?}", type_id))
            })?;

            let search_fn = registry.get_search_fn(*type_id).ok_or_else(|| {
                QueryError::missing_dep(format!("Missing search function for {}", entry.type_name))
            })?;

            // Collect dep nodes (already built due to topo order)
            let deps: Vec<Arc<ExecutionNode>> = entry
                .dependencies
                .iter()
                .filter_map(|dep_id| node_map.get(dep_id).cloned())
                .collect();

            let node = Arc::new(ExecutionNode {
                type_id: *type_id,
                type_name: entry.type_name,
                search_fn,
                cas_runner: AtomicBool::new(false),
                deps,
            });

            node_map.insert(*type_id, node);
        }

        // Get root node
        let root = node_map
            .get(&root_type_id)
            .cloned()
            .ok_or_else(|| QueryError::missing_dep("Root pattern not found in registry"))?;

        // Collect all nodes in topo order
        let nodes: Vec<Arc<ExecutionNode>> = topo_order
            .iter()
            .filter_map(|tid| node_map.get(tid).cloned())
            .collect();

        Ok(Self { root, nodes })
    }

    /// Execute the plan and return a Store with all results.
    ///
    /// # Arguments
    ///
    /// * `driver` - The driver for design access
    /// * `key` - The driver key for the design to search
    /// * `config` - Execution configuration
    pub fn execute(
        &self,
        driver: &Driver,
        key: DriverKey,
        config: &svql_common::Config,
    ) -> Result<Store, QueryError> {
        // Create slots for each node
        let slots: HashMap<TypeId, TableSlot> = self
            .nodes
            .iter()
            .map(|node| (node.type_id, OnceLock::new()))
            .collect();

        // Create shared context
        let ctx = ExecutionContext {
            driver,
            design_key: key,
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
        self.nodes
            .par_iter()
            .try_for_each(|node| self.execute_node(node, ctx))?;

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

        if !node.try_execute() {
            // Another thread is running this dep; wait for it to complete
            if let Some(slot) = ctx.slots.get(&node.type_id()) {
                // Wait for the slot to be filled
                let _ = slot.wait();
            }
            return Ok(());
        }

        // await all dependencies
        for dep in &node.deps {
            // will not re-execute due to cas_runner, and will just block until done
            self.execute_node(dep, ctx)?;
        }

        // Execute search
        let result = (node.search_fn)(ctx)?;

        // Store result wrapped in Arc (OnceLock ensures single write)
        if let Some(slot) = ctx.slots.get(&node.type_id) {
            let _ = slot.set(Arc::from(result)); // Ignore if already set (race condition)
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
    design_key: DriverKey,
    /// Execution configuration.
    config: svql_common::Config,
    /// Slots for storing results (shared with plan).
    slots: Arc<HashMap<TypeId, TableSlot>>,
}

impl<'d> ExecutionContext<'d> {
    /// Get the driver.
    pub fn driver(&self) -> &'d Driver {
        self.driver
    }

    /// Get the driver key.
    pub fn design_key(&self) -> DriverKey {
        self.design_key.clone()
    }

    /// Get the configuration.
    pub fn config(&self) -> &svql_common::Config {
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
            .and_then(|arc| arc.as_any().downcast_ref())
    }

    /// Convert the context into a Store after execution completes.
    fn into_store(self) -> Store {
        let mut store = Store::with_capacity(self.slots.len());

        // Clone tables from slots into the store
        // Since we use Arc<dyn AnyTable>, we can clone the Arc and then
        // use Arc::try_unwrap or just clone the inner table
        for (&type_id, slot) in self.slots.iter() {
            if let Some(arc_table) = slot.get() {
                // Clone the Arc and then box it for the store
                // This is a cheap reference count bump
                store.insert_arc(type_id, Arc::clone(arc_table));
            }
        }

        store
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_config_defaults() {
        let parallel = svql_common::Config::parallel();
        assert!(parallel.parallel);
        assert!(parallel.max_threads.is_none());

        let sequential = !svql_common::Config::parallel();
        assert!(!sequential.parallel);
        assert_eq!(sequential.max_threads, Some(1));
    }
}
