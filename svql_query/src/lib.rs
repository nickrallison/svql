//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.
#![feature(const_type_name)]
#![feature(const_cmp)]
#![feature(const_trait_impl)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::self_only_used_in_recursion)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::needless_continue)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::stable_sort_primitive)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_range_loop)]

extern crate self as svql_query;

// pub mod binding;
// pub mod instance;
pub mod cell_id;
pub mod prelude;
pub mod selector;
// pub mod report;
pub mod session;
pub mod traits;
pub mod wire;

pub use cell_id::CellId;
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

#[must_use] 
pub fn cell_id_to_wrapper(
    cell_id: CellId,
    design_container: &DesignContainer,
) -> Option<CellWrapper<'_>> {
    let index = design_container.index();
    let cell_wrapper = index.get_cell_by_id(cell_id.as_usize())?.clone();
    Some(cell_wrapper)
}
