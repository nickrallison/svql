//! Execution plan and context for parallel pattern matching.
//!
//! The execution model has two phases:
//! 1. **Plan construction** (single-threaded): Build a DAG from the registry
//! 2. **Execution** (multi-threaded): Traverse DAG with OnceLock per slot
//!
//! This module provides the infrastructure. The actual `search` function
//! pointers are provided by the `Pattern` trait implementations.

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
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
    pub nested_dependancies: &'static [&'static ExecInfo],
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
    deps: Vec<Arc<ExecutionNode>>,
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

    fn flatten_deps(&self) -> Vec<Arc<ExecutionNode>> {
        let mut seen: HashSet<TypeId> = HashSet::new();
        let mut result: Vec<Arc<ExecutionNode>> = Vec::new();

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
            deps.push(Arc::new(ExecutionNode::from_dep(nested)));
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
    pub fn build(root: &super::ExecInfo) -> (Self, HashMap<TypeId, TableSlot>) {
        let root_node = Arc::new(ExecutionNode::from_dep(root));
        let mut all_deps = root_node.flatten_deps();
        all_deps.push(Arc::clone(&root_node));
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
        slots: HashMap<TypeId, TableSlot>,
    ) -> Result<Store, QueryError> {
        // Create shared context
        let ctx = ExecutionContext::new(driver.clone(), key.clone(), config.clone(), slots);

        if config.parallel {
            self.execute_parallel(&ctx)?;
        } else {
            self.execute_sequential(&ctx)?;
        }

        // Collect results into Store
        self.try_into_store(&ctx)
    }

    /// Execute nodes sequentially in topological order.
    fn execute_sequential(&self, ctx: &ExecutionContext) -> Result<(), QueryError> {
        for node in &self.nodes {
            self.execute_node(node, ctx)?;
        }
        Ok(())
    }

    /// Execute nodes in parallel using rayon.
    fn execute_parallel(&self, ctx: &ExecutionContext) -> Result<(), QueryError> {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        // Execute all nodes - OnceLock ensures each runs exactly once
        #[cfg(feature = "parallel")]
        let mut iter = self.nodes.par_iter();

        #[cfg(not(feature = "parallel"))]
        let mut iter = self.nodes.iter();

        iter.try_for_each(|node| self.execute_node(node, ctx))?;

        Ok(())
    }

    /// Execute a single node, waiting for deps first.
    fn execute_node(
        &self,
        node: &Arc<ExecutionNode>,
        ctx: &ExecutionContext,
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

    /// Get a dependency's table.
    ///
    /// Returns `None` if the dependency hasn't completed yet or doesn't exist.
    ///
    /// # Panics
    ///
    /// This should only be called for declared dependencies. If DAG ordering
    /// is correct, the dependency will always be available.
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
        for (&type_id, slot) in ctx.slots.iter() {
            if let Some(arc_table) = slot.get() {
                // Clone the Arc and then box it for the store
                // This is a cheap reference count bump
                store.insert_arc(type_id, Arc::clone(arc_table));
            } else {
                return Err(QueryError::ExecutionError(format!(
                    "Missing table for type_id: {:?}",
                    type_id
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
    /// Execution configuration.
    config: svql_common::Config,
    /// Slots to hold results during execution.
    slots: HashMap<TypeId, TableSlot>,
}

impl ExecutionContext {
    pub fn new(
        driver: Driver,
        design_key: DriverKey,
        config: svql_common::Config,
        slots: HashMap<TypeId, TableSlot>,
    ) -> Self {
        Self {
            driver,
            design_key,
            config,
            slots,
        }
    }

    /// Get the driver.
    pub fn driver(&self) -> &Driver {
        &self.driver
    }

    /// Get the driver key.
    pub fn design_key(&self) -> DriverKey {
        self.design_key.clone()
    }

    /// Get the configuration.
    pub fn config(&self) -> &svql_common::Config {
        &self.config
    }

    /// Retrieve a completed dependency table by TypeId.
    ///
    /// Returns `None` if the table was not found or is not yet computed
    /// (though DAG ordering guarantees it should be computed).
    pub fn get_any_table(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.slots
            .get(&type_id)
            .and_then(|slot| slot.get())
            .map(|arc| arc.as_ref())
    }
}
