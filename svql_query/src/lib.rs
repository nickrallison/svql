//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.
#![feature(const_type_name)]

extern crate self as svql_query;

// pub mod binding;
// pub mod instance;
pub mod prelude;
// pub mod report;
pub mod session;
pub mod traits;
pub mod wire;

pub use wire::Wire;
pub mod driver {
    pub use svql_driver::*;
}

#[macro_use]
pub mod test_harness;

pub use svql_common as common;
use svql_driver::design_container::DesignContainer;
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
    config: &svql_common::Config,
) -> Result<session::Store, Box<dyn std::error::Error>>
where
    P: Pattern + Send + Sync + 'static,
{
    P::search(driver, key, config).map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
}

pub fn cell_id_to_wrapper<'a>(
    cell_id: u64,
    design_container: &'a DesignContainer,
) -> Option<CellWrapper<'a>> {
    let index = design_container.index();
    let cell_wrapper = index.get_cell_by_id(cell_id as usize)?.clone();
    Some(cell_wrapper)
}

/// Represents a connection between two wires.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    pub from: u64,
    pub to: u64,
}
