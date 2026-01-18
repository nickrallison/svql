//! Direct dehydration during search - avoiding Match object allocation.
//!
//! This module provides the `SearchDehydrate` trait which enables search types
//! to directly produce `DehydratedRow`s during the search process, bypassing
//! the intermediate `Match` object allocation.

use svql_driver::{Context, Driver, DriverKey};
use crate::common::Config;
use crate::traits::SearchableComponent;
use super::{DehydratedResults, QuerySchema};

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
pub trait SearchDehydrate: SearchableComponent {
    /// The schema for the dehydrated Match type.
    const MATCH_SCHEMA: QuerySchema;

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
