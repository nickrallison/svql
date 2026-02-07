//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.
#![feature(const_type_name)]
#![feature(const_cmp)]
#![feature(const_trait_impl)]

extern crate self as svql_query;

pub mod dsl;
pub mod prelude;
pub mod session;

#[macro_use]
pub mod test_harness;

use prelude::*;

// Re-export for macro usage
pub use svql_common as common;
pub use svql_common::Wire;
pub use svql_driver as driver;

// Maintain backward compatibility for macros
pub use dsl::traits;
pub use dsl::selector;

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
/// let store = run_query_with_config::<MyPattern>(&driver, &key, config)?;
///
/// // Or limit threads
/// let config = ExecConfig { parallel: true, max_threads: Some(4) };
/// let store = run_query_with_config::<MyPattern>(&driver, &key, config)?;
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
