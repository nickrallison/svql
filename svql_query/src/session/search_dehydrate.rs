//! Direct dehydration during search - avoiding Match object allocation.
//!
//! This module provides the `SearchDehydrate` trait which enables search types
//! to directly produce `DehydratedRow`s during the search process, bypassing
//! the intermediate `Match` object allocation.
//!
//! # Deprecation Notice
//!
//! This module is deprecated and will be removed in a future version.
//! Use the new `ExecutionPlan` API instead:
//!
//! ```ignore
//! // Instead of:
//! let results = DehydratedResults::new();
//! search.execute_dehydrated(driver, context, key, config, &mut results);
//!
//! // Use:
//! let plan = ExecutionPlan::for_pattern::<T>();
//! let store = plan.execute(driver, context, key, config)?;
//! ```

use super::{DehydratedResults, QuerySchema};
use crate::common::Config;
use crate::traits::SearchableComponent;
use svql_driver::{Context, Driver, DriverKey};

/// Trait for Search types that can directly produce dehydrated results.
///
/// This is the key trait for efficient query execution - instead of:
/// 1. Search → Vec<Match>
/// 2. Vec<Match> → Vec<DehydratedRow>
/// 3. Vec<DehydratedRow> → DataFrame
///
/// We do:
/// 1. Search → DehydratedResults (directly populates DataFrames)
///
/// This avoids allocating intermediate Match objects entirely.
///
/// **Deprecated:** Use `ExecutionPlan` instead.
#[deprecated(since = "0.2.0", note = "Use ExecutionPlan instead")]
#[allow(deprecated)]
pub trait SearchDehydrate: SearchableComponent {
    /// The schema for the dehydrated Match type.
    const MATCH_SCHEMA: QuerySchema;

    /// Returns the full type path for storage/lookup.
    ///
    /// Uses `std::any::type_name` to get a unique key that includes
    /// the full module path, avoiding collisions between types with
    /// the same simple name (e.g., variants vs their inner types).
    fn type_key() -> &'static str {
        std::any::type_name::<Self::Match>()
    }

    /// Executes the search and directly produces dehydrated results.
    ///
    /// The results are accumulated into the provided `DehydratedResults`,
    /// which allows collecting results for this type and all submodule types
    /// in a single pass.
    ///
    /// Returns the indices of the newly added rows for this type.
    fn execute_dehydrated(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
        results: &mut DehydratedResults,
    ) -> Vec<u32>;
}
