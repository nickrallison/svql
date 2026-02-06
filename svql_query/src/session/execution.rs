//! Execution plan and context for parallel pattern matching.
//!
//! The execution model has two phases:
//! 1. **Plan construction** (single-threaded): Build a DAG from the registry
//! 2. **Execution** (multi-threaded): Traverse DAG with `OnceLock` per slot
//!
//! This module provides the infrastructure. The actual `search` function
//! pointers are provided by the `Pattern` trait implementations.

use ahash::{AHashMap, AHashSet};
use std::any::{Any, TypeId};
use std::hash::Hash;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use crate::prelude::*;

/// Type alias for a search function.
///
/// Search functions take an `ExecutionContext` and return a type-erased table.
/// They are provided by `Pattern::search()` implementations.
pub type SearchFn = fn(&ExecutionContext) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>;

pub struct ExecInfo {
    pub type_id: std::any::TypeId,
    pub type_name: &'static str,
    pub search_function: SearchFn,
    pub nested_dependancies: &'static [&'static Self],
}

/// Slot for storing a table during execution.
///
/// Uses `OnceLock<Arc<dyn AnyTable>>` so tables can be shared during execution
/// and then cloned into the final Store.
type TableSlot = OnceLock<Arc<dyn AnyTable + Send + Sync>>;

/// A node in the execution DAG.
#[derive(Debug)]
struct ExecutionNode {
    /// The pattern type this node represents.
    type_id: TypeId,
    /// Human-readable name for debugging.
    type_name: &'static str,
    /// The search function to execute.
    search_fn: SearchFn,
    /// Atomic flag to ensure single execution in parallel mode.
    cas_runner: AtomicBool,
    /// Dependencies that must complete before this node.
    deps: Vec<Arc<Self>>,
}

impl ExecutionNode {
    // /// Check if this node has already been executed.
    // fn is_executed(&self) -> bool {
    //     self.cas_runner.load(std::sync::atomic::Ordering::SeqCst)
    // }

    // fn is_done(&self, ctx: &ExecutionContext) -> bool {
    //     if let Some(slot) = ctx.slots.get(&self.type_id)
    //         && slot.get().is_some()
    //     {
    //         return true;
    //     }
    //     false
    // }

    fn try_execute(&self) -> bool {
        let cas_result = self.cas_runner.compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        );
        cas_result.is_ok()
    }

    fn flatten_deps(&self) -> Vec<Arc<Self>> {
        let mut seen: AHashSet<TypeId> = AHashSet::new();
        let mut result: Vec<Arc<Self>> = Vec::new();

        for dep in &self.deps {
            if seen.insert(dep.type_id) {
                result.push(Arc::clone(dep));
            }

            // Recursively flatten and deduplicate
            for nested in dep.flatten_deps() {
                if seen.insert(nested.type_id) {
                    result.push(nested);
                }
            }
        }

        result
    }

    fn from_dep(exec_info: &ExecInfo) -> Self {
        let mut deps = vec![];
        for nested in exec_info.nested_dependancies {
            deps.push(Arc::new(Self::from_dep(nested)));
        }
        Self {
            type_id: exec_info.type_id,
            type_name: exec_info.type_name,
            search_fn: exec_info.search_function,
            cas_runner: AtomicBool::new(false),
            deps,
        }
    }
}

impl PartialEq for ExecutionNode {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for ExecutionNode {}

impl Hash for ExecutionNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

/// An execution plan built from a pattern's dependency graph.
///
/// The plan contains all nodes needed to execute a pattern query,
/// with dependencies properly ordered.
pub struct ExecutionPlan {
    /// The root node (the pattern we're ultimately searching for).
    root: Arc<ExecutionNode>,
    /// All nodes in topological order (deps before dependents).
    nodes: Vec<Arc<ExecutionNode>>,
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
    #[must_use]
    pub fn build(root: &super::ExecInfo) -> (Self, AHashMap<TypeId, TableSlot>) {
        tracing::info!("Building execution plan for pattern: {}", root.type_name);
        let root_node = Arc::new(ExecutionNode::from_dep(root));
        let mut all_deps = root_node.flatten_deps();
        all_deps.push(Arc::clone(&root_node));
        
        tracing::debug!("Execution plan has {} nodes total", all_deps.len());
        for (i, node) in all_deps.iter().enumerate() {
            tracing::trace!("  Node {}: {} (deps: {})", i, node.type_name, node.deps.len());
        }
        
        let slots = all_deps
            .iter()
            .map(|node| (node.type_id, OnceLock::new()))
            .collect();
        (
            Self {
                root: root_node,
                nodes: all_deps,
            },
            slots,
        )
    }

    /// Execute the plan and return a Store with all results.
    ///
    /// # Arguments
    ///
    /// * `driver` - The driver for design access
    /// * `key` - The driver key for the design to search
    /// * `config` - Execution configuration
    pub fn execute(
        self,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
        slots: AHashMap<TypeId, TableSlot>,
    ) -> Result<Store, QueryError> {
        tracing::info!("Starting execution plan for design: {:?}", key);
        tracing::info!("Execution mode: {}", if config.parallel { "parallel" } else { "sequential" });
        
        // Load and cache the haystack design once
        tracing::debug!("Loading haystack design...");
        let haystack_design = driver
            .get_design(key, &config.haystack_options)
            .map_err(|e| QueryError::design_load(e.to_string()))?;
        tracing::debug!("Haystack design loaded successfully");

        // Create shared context
        let ctx = ExecutionContext::new(
            driver.clone(),
            key.clone(),
            haystack_design,
            config.clone(),
            slots,
        );

        if config.parallel {
            tracing::info!("Executing plan in parallel mode");
            self.execute_parallel(&ctx)?;
        } else {
            tracing::info!("Executing plan in sequential mode");
            self.execute_sequential(&ctx)?;
        }

        tracing::info!("Plan execution complete, collecting results...");
        // Collect results into Store
        let store = self.try_into_store(&ctx)?;
        tracing::info!("Store created with {} tables", store.len());
        Ok(store)
    }

    /// Execute nodes sequentially in topological order.
    fn execute_sequential(&self, ctx: &ExecutionContext) -> Result<(), QueryError> {
        tracing::debug!("Executing {} nodes sequentially", self.nodes.len());
        for (i, node) in self.nodes.iter().enumerate() {
            tracing::debug!("[{}/{}] Executing node: {}", i + 1, self.nodes.len(), node.type_name);
            ExecutionPlan::execute_node(node, ctx)?;
        }
        tracing::debug!("Sequential execution complete");
        Ok(())
    }

    /// Execute nodes in parallel using rayon.
    fn execute_parallel(&self, ctx: &ExecutionContext) -> Result<(), QueryError> {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        // Execute all nodes - OnceLock ensures each runs exactly once
        #[cfg(feature = "parallel")]
        self.nodes
            .par_iter()
            .try_for_each(|node| ExecutionPlan::execute_node(node, ctx))?;

        #[cfg(not(feature = "parallel"))]
        self.nodes
            .iter()
            .try_for_each(|node| ExecutionPlan::execute_node(node, ctx))?;

        Ok(())
    }

    /// Execute a single node, waiting for deps first.
    fn execute_node(node: &Arc<ExecutionNode>, ctx: &ExecutionContext) -> Result<(), QueryError> {
        // Check if already executed
        tracing::trace!("Attempting to execute node: {}", node.type_name);

        if !node.try_execute() {
            tracing::trace!("Node {} already executing or completed, waiting...", node.type_name);
            // Another thread is running this dep; wait for it to complete
            if let Some(slot) = ctx.slots.get(&node.type_id()) {
                // Wait for the slot to be filled
                let _ = slot.wait();
            }
            tracing::trace!("Node {} execution complete (waited)", node.type_name);
            return Ok(());
        }

        tracing::debug!("Executing node: {} (with {} dependencies)", node.type_name, node.deps.len());
        
        // await all dependencies
        for (i, dep) in node.deps.iter().enumerate() {
            tracing::trace!("  Waiting for dependency {}/{}: {}", i + 1, node.deps.len(), dep.type_name);
            // will not re-execute due to cas_runner, and will just block until done
            ExecutionPlan::execute_node(dep, ctx)?;
            tracing::trace!("  Dependency {}/{} complete: {}", i + 1, node.deps.len(), dep.type_name);
        }

        // Execute search
        tracing::info!("[SEARCH] Starting search for: {}", node.type_name);
        let result = (node.search_fn)(ctx)?;
        tracing::info!("[SEARCH] Completed search for: {} -> {} results", node.type_name, result.len());

        // Store result wrapped in Arc (OnceLock ensures single write)
        if let Some(slot) = ctx.slots.get(&node.type_id) {
            let _ = slot.set(Arc::from(result)); // Ignore if already set (race condition)
            tracing::trace!("Stored results for: {}", node.type_name);
        }

        tracing::debug!("Node execution complete: {}", node.type_name);
        Ok(())
    }

    /// Get a dependency's table.
    ///
    /// Returns `None` if the dependency hasn't completed yet or doesn't exist.
    ///
    /// # Panics
    ///
    /// This should only be called for declared dependencies. If DAG ordering
    /// is correct, the dependency will always be available.
    #[must_use]
    pub fn get_table<'a, T: 'static>(
        &self,
        ctx: &'a ExecutionContext,
    ) -> Option<&'a super::table::Table<T>> {
        ctx.slots
            .get(&TypeId::of::<T>())
            .and_then(|lock| lock.get())
            .and_then(|arc| arc.as_any().downcast_ref())
    }

    /// Convert the context into a Store after execution completes.
    pub fn try_into_store(self, ctx: &ExecutionContext) -> Result<Store, QueryError> {
        let mut store = Store::with_capacity(ctx.slots.len());

        // Clone tables from slots into the store
        // Since we use Arc<dyn AnyTable>, we can clone the Arc and then
        // use Arc::try_unwrap or just clone the inner table
        for (&type_id, slot) in &ctx.slots {
            if let Some(arc_table) = slot.get() {
                // Clone the Arc and then box it for the store
                // This is a cheap reference count bump
                store.insert_arc(type_id, Arc::clone(arc_table));
            } else {
                return Err(QueryError::ExecutionError(format!(
                    "Missing table for type_id: {type_id:?}"
                )));
            }
        }

        Ok(store)
    }
}

/// Context passed to pattern search functions.
///
/// Provides access to:
/// - The driver for design access
/// - The driver key for the current design
/// - Completed dependency tables
pub struct ExecutionContext {
    /// The driver for design/needle operations.
    driver: Driver,
    /// Key for the design being searched.
    design_key: DriverKey,
    /// Cached haystack design container to avoid repeated get_design calls.
    haystack_design: Arc<svql_driver::design_container::DesignContainer>,
    /// Execution configuration.
    config: svql_common::Config,
    /// Slots to hold results during execution.
    slots: AHashMap<TypeId, TableSlot>,
}

impl ExecutionContext {
    #[must_use]
    pub fn new(
        driver: Driver,
        design_key: DriverKey,
        haystack_design: Arc<svql_driver::design_container::DesignContainer>,
        config: svql_common::Config,
        slots: AHashMap<TypeId, TableSlot>,
    ) -> Self {
        #[cfg(not(feature = "parallel"))]
        if config.parallel {
            tracing::warn!(
                "Parallel execution requested but 'parallel' feature is not enabled. Falling back to sequential execution."
            );
        }

        Self {
            driver,
            design_key,
            haystack_design,
            config,
            slots,
        }
    }

    /// Get the driver.
    #[must_use]
    pub const fn driver(&self) -> &Driver {
        &self.driver
    }

    /// Get the driver key.
    #[must_use]
    pub fn design_key(&self) -> DriverKey {
        self.design_key.clone()
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &svql_common::Config {
        &self.config
    }

    /// Get the cached haystack design container.
    #[must_use]
    pub fn haystack_design(&self) -> &Arc<svql_driver::design_container::DesignContainer> {
        &self.haystack_design
    }

    /// Retrieve a completed dependency table by `TypeId`.
    ///
    /// Returns `None` if the table was not found or is not yet computed
    /// (though DAG ordering guarantees it should be computed).
    #[must_use]
    pub fn get_any_table(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.slots
            .get(&type_id)
            .and_then(|slot| slot.get())
            .map(std::convert::AsRef::as_ref)
    }
}
