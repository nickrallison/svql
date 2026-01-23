//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.
// #![feature(generic_const_exprs)]
// #![feature(associated_type_defaults)]

extern crate self as svql_query;

// pub mod binding;
pub mod instance;
pub mod prelude;
pub mod report;
pub mod session;
pub mod traits;

/// Re-export of driver crate with Context flattened for macro compatibility
pub mod driver {
    pub use svql_driver::*;
}

pub use svql_common as common;
pub use svql_subgraph as subgraph;

use prelude::*;

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
pub fn run_query<P>(
    driver: &Driver,
    key: &DriverKey,
    config: svql_common::Config,
) -> Result<session::Store, Box<dyn std::error::Error>>
where
    P: Pattern + Send + Sync + 'static,
{
    let plan = session::ExecutionPlan::for_pattern::<P>()?;
    let store = plan.execute(driver, key.clone(), &config)?;

    Ok(store)
}

/// Represents a connection between two wires.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    pub from: CellId,
    pub to: CellId,
}
