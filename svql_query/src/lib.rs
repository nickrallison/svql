//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.

extern crate self as svql_query;

pub mod binding;
pub mod composites;
pub mod instance;
pub mod prelude;
pub mod primitives;
pub mod report;
pub mod security;
pub mod session;
pub mod traits;
pub mod variants;

/// Re-export of driver crate with Context flattened for macro compatibility
pub mod driver {
    pub use svql_driver::context::Context;
    pub use svql_driver::*;
}

pub use svql_common as common;
pub use svql_subgraph as subgraph;

use prelude::*;
use traits::Hardware;

/// A high-level helper to execute a query by type.
///
/// This handles:
/// 1. Loading the design (if not loaded).
/// 2. Building the specific Context for this query.
/// 3. Instantiating the query root.
/// 4. Running the search.
// pub fn execute_query<P>(
//     driver: &Driver,
//     key: &DriverKey,
//     config: &Config,
// ) -> Result<Vec<P::Match>, Box<dyn std::error::Error>>
// where
//     P: traits::Pattern + 'static,
// {
//     // 1. Build the Context
//     let needle_ctx = P::context(driver, &config.needle_options)?;

//     // 2. Add the Haystack to the context
//     let design_container = driver
//         .get_design(key)
//         .ok_or_else(|| format!("Design not found in driver: {:?}", key))?;

//     let context = needle_ctx.with_design(key.clone(), design_container);

//     // 3. Instantiate the Query Root
//     let root_name = std::any::type_name::<P>()
//         .split("::")
//         .last()
//         .unwrap_or("query")
//         .to_lowercase();
//     let query_instance = P::instantiate(Instance::root(root_name));

//     // 4. Execute
//     let results = query_instance.execute(driver, &context, key, config);

//     Ok(results)
// }

/// Execute a query and return results directly in a Session with DataFrame storage.
///
/// This is the preferred method for production use as it:
/// - Dehydrates matches directly into columnar storage
/// - Avoids allocating full Match objects
/// - Enables lazy rehydration when individual matches are needed
///
/// The returned Session contains:
/// - The design data as a DesignFrame
/// - Query results as DataFrames with foreign key references
// pub fn execute_query_session<P>(
//     driver: &Driver,
//     key: &DriverKey,
//     config: &Config,
// ) -> Result<session::Session, Box<dyn std::error::Error>>
// where
//     P: traits::Pattern + 'static,
//     P::Match: session::Dehydrate,
// {
//     // 1. Build the Context
//     let needle_ctx = P::context(driver, &config.needle_options)?;

//     // 2. Add the Haystack to the context
//     let design_container = driver
//         .get_design(key)
//         .ok_or_else(|| format!("Design not found in driver: {:?}", key))?;

//     let context = needle_ctx.with_design(key.clone(), design_container.clone());

//     // 3. Instantiate the Query Root
//     let root_name = std::any::type_name::<P>()
//         .split("::")
//         .last()
//         .unwrap_or("query")
//         .to_lowercase();
//     let query_instance = P::instantiate(Instance::root(root_name));

//     // 4. Execute and get matches
//     let matches = query_instance.execute(driver, &context, key, config);

//     // 5. Dehydrate into session
//     let dehydrated = session::Dehydrate::dehydrate_all(&matches)?;
//     let session = session::SessionBuilder::new(design_container)
//         .with_results::<P>(dehydrated)
//         .build()?;

//     Ok(session)
// }

// /// Execute a query and return results directly in a Session without allocating Match objects.
// ///
// /// This is the most efficient method for query execution as it:
// /// - Directly produces dehydrated rows during search
// /// - Never allocates intermediate Match objects
// /// - Stores all results in efficient columnar storage
// ///
// /// The returned Session contains:
// /// - The design data as a DesignFrame
// /// - Query results for all types (main query + submodules) as DataFrames
// pub fn execute_query_session_direct<P>(
//     driver: &Driver,
//     key: &DriverKey,
//     config: &Config,
// ) -> Result<session::Session, Box<dyn std::error::Error>>
// where
//     P: traits::Pattern + session::SearchDehydrate + 'static,
//     <P as traits::Pattern>::Match: session::Dehydrate,
// {
//     // 1. Build the Context
//     let needle_ctx = P::context(driver, &config.needle_options)?;

//     // 2. Add the Haystack to the context
//     let design_container = driver
//         .get_design(key)
//         .ok_or_else(|| format!("Design not found in driver: {:?}", key))?;

//     let context = needle_ctx.with_design(key.clone(), design_container.clone());

//     // 3. Instantiate the Query Root
//     let root_name = std::any::type_name::<P>()
//         .split("::")
//         .last()
//         .unwrap_or("query")
//         .to_lowercase();
//     let query_instance = P::instantiate(Instance::root(root_name));

//     // 4. Execute directly into dehydrated results
//     let mut results = session::DehydratedResults::new();
//     let _ = query_instance.execute_dehydrated(driver, &context, key, config, &mut results);

//     // 5. Build session from dehydrated results
//     let query_results_map = results.into_query_results()?;
//     let session = session::SessionBuilder::new(design_container)
//         .with_results_map::<P>(query_results_map)
//         .build()?;

//     Ok(session)
// }

// ============================================================================
// New DataFrame-based API (recommended)
// ============================================================================

/// Execute a query and return results in a Store with DataFrame storage.
///
/// This is the new, preferred API that uses the DataFrame-based execution model:
/// - Uses `ExecutionPlan` for DAG-based parallel execution
/// - Automatically registers all pattern dependencies
/// - Results stored in `Table<T>` with column-oriented storage
/// - Supports lazy iteration via `Table::rows()`
/// - Type-safe references via `Ref<T>`
///
/// Works for all pattern types:
/// - **Netlist patterns**: Atomic patterns loaded from HDL files
/// - **Composite patterns**: Hierarchical patterns with sub-components
/// - **Variant patterns**: Polymorphic patterns that match multiple alternatives
///
/// # Example
///
/// ```ignore
/// use svql_query::prelude::*;
///
/// // Works with any pattern type
/// let store = run_query::<MyCompositePattern<Search>>(&driver, &key)?;
///
/// // Access the root pattern's results
/// let table = store.get::<MyCompositePattern<Search>>()?;
///
/// for row in table.rows() {
///     let wire_cell = row.wire("clk");
///     // Access sub-component results via typed references
///     let sub_ref: Ref<SubPattern<Search>> = row.sub("sub_field")?;
/// }
///
/// // Dependency tables are also available
/// let dep_table = store.get::<SubPattern<Search>>()?;
/// ```
pub fn run_query<P>(
    driver: &Driver,
    key: &DriverKey,
) -> Result<session::Store, Box<dyn std::error::Error>>
where
    P: traits::SearchableComponent + Send + Sync + 'static,
{
    use session::{Config as ExecConfig, ExecutionPlan};

    // Build execution plan from pattern (registers all dependencies)
    let plan = ExecutionPlan::for_pattern::<P>()?;

    // Execute with default config
    let config = ExecConfig::sequential(); // Start with sequential for simplicity
    let store = plan.execute(driver, key.clone(), config)?;

    Ok(store)
}

/// Execute a query with custom execution configuration.
///
/// Allows specifying parallel vs sequential execution and thread limits.
///
/// # Example
///
/// ```ignore
/// use svql_query::prelude::*;
///
/// // Run in parallel with default thread count
/// let config = ExecConfig::parallel();
/// let store = run_query_with_config::<MyPattern<Search>>(&driver, &key, config)?;
///
/// // Or limit threads
/// let config = ExecConfig { parallel: true, max_threads: Some(4) };
/// let store = run_query_with_config::<MyPattern<Search>>(&driver, &key, config)?;
/// ```
pub fn run_query_with_config<P>(
    driver: &Driver,
    key: &DriverKey,
    config: session::Config,
) -> Result<session::Store, Box<dyn std::error::Error>>
where
    P: traits::SearchableComponent + Send + Sync + 'static,
{
    use session::ExecutionPlan;

    let plan = ExecutionPlan::for_pattern::<P>()?;
    let store = plan.execute(driver, key.clone(), config)?;

    Ok(store)
}

/// Defines the state of a query component.
///
/// Components exist in two primary states:
/// 1. `Search`: The initial state where components define the pattern to find.
/// 2. `Match`: The result state where components hold references to design elements.
pub trait State: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash {
    /// The internal data type held by a Wire in this state.
    type WireInner: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash;

    /// Helper to extract source location from the inner wire type.
    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        let _ = inner;
        None
    }
}

/// Represents a query in its search/definition phase.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Search;

impl State for Search {
    type WireInner = ();
}

/// Represents a query result bound to specific design elements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Match;

impl State for Match {
    type WireInner = Option<CellInfo>;

    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        inner.as_ref().and_then(|cell| cell.get_source())
    }
}

/// A logical connection point within a query component.
///
/// `Wire` is a simple data container, not a searchable pattern.
/// In `Search` state, it holds only a path. In `Match` state,
/// it additionally holds information about the matched design cell.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Wire<S: State>
where
    S::WireInner: Clone + std::fmt::Debug + PartialEq + Eq,
{
    /// Hierarchical path of the wire.
    pub path: Instance,
    /// State-specific data (empty for Search, CellInfo for Match).
    pub inner: S::WireInner,
}

impl<S: State> Wire<S> {
    /// Creates a new wire instance.
    pub fn new(path: Instance, inner: S::WireInner) -> Self {
        Self { path, inner }
    }

    /// Returns the hierarchical path of the wire.
    pub fn path(&self) -> &Instance {
        &self.path
    }
}

impl Wire<Search> {
    /// Creates a search wire at the given path.
    pub fn search(path: Instance) -> Self {
        Self::new(path, ())
    }
}

impl Wire<Match> {
    /// Returns the matched cell associated with this wire.
    pub fn cell(&self) -> Option<&subgraph::cell::CellInfo> {
        self.inner.as_ref()
    }

    /// Creates a match wire with the given cell info.
    pub fn matched(path: Instance, cell: Option<CellInfo>) -> Self {
        Self::new(path, cell)
    }
}

/// Hardware implementation for Wire.
///
/// Wire implements Hardware so it can be returned in `children()` trait object vectors,
/// but it is NOT a searchable component - it's just a data container.
impl<S: State> Hardware for Wire<S> {
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "Wire"
    }

    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        Vec::new() // Wires are leaf nodes
    }

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if path == &self.path { Some(self) } else { None }
    }

    fn source(&self) -> Option<SourceLocation> {
        S::wire_source(&self.inner)
    }
}

/// Represents a connection between two wires.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection<S>
where
    S: State,
{
    pub from: Wire<S>,
    pub to: Wire<S>,
}
