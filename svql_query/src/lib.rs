//! Core query engine and result storage for SVQL.
//!
//! The `svql_query` crate provides:
//! - **DSL**: Trait definitions for patterns (`Netlist`, `Composite`, `Primitive`, `Variant`, `Recursive`)
//! - **Session**: Result storage with columnar tables, schemas, and execution planning
//! - **Pattern Search**: Entry point via `run_query()` for executing pattern matches
//!
//! # Query Execution Flow
//!
//! 1. User calls `run_query::<P>(&driver, &key, &config)?`
//! 2. Pattern `P` defines its structure (components, connections, filters)
//! 3. Execution plan is built from pattern dependencies
//! 4. Subgraph matcher finds all matches in the design
//! 5. Results are stored in `Store` with type-safe tables
//! 6. Patterns can render results for detailed reporting

#![feature(const_type_name)]

extern crate self as svql_query;

pub mod dsl;
/// Common types and re-exports for building queries.
pub mod prelude;
pub mod session;

/// Testing utilities and fixture orchestration.
#[macro_use]
pub mod test_harness;

use prelude::*;

pub use svql_common as common;
pub use svql_common::Wire;
pub use svql_driver as driver;

pub use dsl::traits;

/// Executes a pattern query against a design.
///
/// This is the primary entry point for pattern matching. It orchestrates:
/// 1. Design loading and caching via the driver
/// 2. Pattern matching according to configuration
/// 3. Result storage in a columnar table format
///
/// # Arguments
///
/// * `driver` - Design loader and cache manager
/// * `key` - Design identifier (path + module name)
/// * `config` - Query configuration (parallelism, match constraints, etc.)
///
/// # Example
///
/// ```ignore
/// use svql_query::prelude::*;
/// use svql_query_lib::security::cwe1234::Cwe1234;
///
/// let driver = Driver::new_workspace()?;
/// let key = DriverKey::new("design.json", "top");
/// let config = Config::builder().parallel(true).build();
/// let store = run_query::<Cwe1234>(&driver, &key, &config)?;
/// let results = store.get::<Cwe1234>().unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if the query execution fails.
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
