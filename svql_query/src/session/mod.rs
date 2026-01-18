//! DataFrame-based session management for query results.
//!
//! This module provides a `Session` struct that replaces the traditional
//! index lookup tables with polars DataFrames, enabling:
//! - Foreign-key style references between result tables
//! - Lazy rehydration of match results
//! - Efficient columnar storage of dehydrated matches

mod design_frame;
mod foreign_key;
mod rehydrate;
mod result_store;
mod search_dehydrate;

pub use design_frame::{CellRow, DesignFrame};
pub use foreign_key::{ForeignKey, ForeignKeyTarget};
pub use rehydrate::{
    MatchRef, Rehydrate, RehydrateContext, RehydrateIter, SessionRehydrateExt, WireRef,
};
pub use result_store::{
    Dehydrate, DehydratedResults, DehydratedRow, MatchRow, QueryResults, QuerySchema,
    RecursiveFieldDesc, ResultStore, SubmoduleFieldDesc, WireFieldDesc,
};
pub use search_dehydrate::SearchDehydrate;

// Re-export validation helper for use by macros
pub use self::validate_dehydrated_connection as validate_connection;

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use polars::prelude::*;
use prjunnamed_netlist::Design;
use svql_driver::design_container::DesignContainer;
use thiserror::Error;

use crate::traits::Pattern;

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Design not loaded: {0}")]
    DesignNotLoaded(String),
    #[error("Query results not found for type: {0}")]
    ResultsNotFound(String),
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),
    #[error("Invalid match index: {0}")]
    InvalidMatchIndex(u32),
    #[error("Rehydration error: {0}")]
    RehydrationError(String),
}

/// A query session that stores designs and results in DataFrames.
///
/// The session provides:
/// - Design data stored as a `DesignFrame` (cells, connectivity as columns)
/// - Query results stored per-type with foreign key references
/// - Lazy rehydration back to full `Match` state objects
#[derive(Debug)]
pub struct Session {
    /// The design being queried, stored in DataFrame format
    design: DesignFrame,
    /// Query results keyed by query type name
    results: ResultStore,
    /// Original design container reference for rehydration
    design_container: Arc<DesignContainer>,
}

impl Session {
    /// Creates a new session from a design container.
    pub fn new(design_container: Arc<DesignContainer>) -> Result<Self, SessionError> {
        let design_frame = DesignFrame::from_design(design_container.design())?;

        Ok(Self {
            design: design_frame,
            results: ResultStore::new(),
            design_container,
        })
    }

    /// Returns a reference to the design DataFrame.
    pub fn design_frame(&self) -> &DesignFrame {
        &self.design
    }

    /// Returns a reference to the result store.
    pub fn results(&self) -> &ResultStore {
        &self.results
    }

    /// Returns a mutable reference to the result store.
    pub fn results_mut(&mut self) -> &mut ResultStore {
        &mut self.results
    }

    /// Returns the original design reference.
    pub fn design(&self) -> &Design {
        self.design_container.design()
    }

    /// Stores dehydrated results for a query type.
    pub fn store_results<P: Pattern + 'static>(
        &mut self,
        results: QueryResults,
    ) -> Result<(), SessionError> {
        self.results.insert::<P>(results);
        Ok(())
    }

    /// Retrieves results for a query type.
    pub fn get_results<P: Pattern + 'static>(&self) -> Option<&QueryResults> {
        self.results.get::<P>()
    }

    /// Creates a rehydration context for lazily converting dehydrated matches.
    pub fn rehydrate_context(&self) -> RehydrateContext<'_> {
        RehydrateContext::new(&self.design, &self.results, self.design_container.design())
    }

    /// Returns the number of cells in the design.
    pub fn num_cells(&self) -> usize {
        self.design.num_cells()
    }

    /// Returns the number of stored query types.
    pub fn num_query_types(&self) -> usize {
        self.results.len()
    }

    /// Gets cell info by cell_id for rehydration.
    pub fn get_cell_info(&self, cell_id: u32) -> Option<CellRow> {
        self.design.get_cell(cell_id)
    }

    /// Checks if the fanout of `from_cell_id` contains `to_cell_id`.
    pub fn is_connected(&self, from_cell_id: u32, to_cell_id: u32) -> bool {
        self.design.is_connected(from_cell_id, to_cell_id)
    }
}

/// Builder for constructing a Session with query results.
pub struct SessionBuilder {
    design_container: Arc<DesignContainer>,
    pending_results: HashMap<TypeId, (String, QueryResults)>,
}

impl SessionBuilder {
    /// Creates a new session builder.
    pub fn new(design_container: Arc<DesignContainer>) -> Self {
        Self {
            design_container,
            pending_results: HashMap::new(),
        }
    }

    /// Adds query results to the session.
    ///
    /// Uses the schema's type_name from `Dehydrate::SCHEMA` for consistent
    /// lookup during rehydration.
    pub fn with_results<P>(mut self, results: QueryResults) -> Self
    where
        P: Pattern + 'static,
        P::Match: Dehydrate,
    {
        let type_name = <P::Match as Dehydrate>::SCHEMA.type_name.to_string();
        self.pending_results
            .insert(TypeId::of::<P>(), (type_name, results));
        self
    }

    /// Adds multiple query results from a HashMap (for direct dehydration).
    ///
    /// This is used when results come from `SearchDehydrate::execute_dehydrated`
    /// which produces results for multiple types (main query + submodules).
    pub fn with_results_map<P>(mut self, results_map: HashMap<String, QueryResults>) -> Self
    where
        P: Pattern + 'static,
        P::Match: Dehydrate,
    {
        // Insert the main query type with its proper TypeId
        // Use full type path for consistent lookup
        let main_type_name = std::any::type_name::<P::Match>().to_string();

        if let Some(main_results) = results_map.get(&main_type_name) {
            self.pending_results.insert(
                TypeId::of::<P>(),
                (main_type_name.clone(), main_results.clone()),
            );
        }

        // Insert submodule results with generated TypeIds (keyed by name)
        for (type_name, results) in results_map {
            if type_name != main_type_name {
                // Use a placeholder TypeId for submodules (they don't have a Rust type)
                // These can only be looked up by name, not by type
                // Note: We store these for name-based lookup, TypeId won't match any real type
                self.pending_results.insert(
                    TypeId::of::<()>(), // Placeholder - these are looked up by name
                    (type_name, results),
                );
            }
        }
        self
    }

    /// Builds the session.
    pub fn build(self) -> Result<Session, SessionError> {
        let mut session = Session::new(self.design_container)?;

        for (type_id, (type_name, results)) in self.pending_results {
            session.results.insert_raw(type_id, type_name, results);
        }

        Ok(session)
    }
}

/// Validates that two cells (by ID) are connected in the design graph.
///
/// This is used by composite SearchDehydrate implementations to validate
/// topology connections between submodule wires.
pub fn validate_dehydrated_connection<'ctx>(
    from_cell_id: Option<u32>,
    to_cell_id: Option<u32>,
    haystack_index: &svql_subgraph::GraphIndex<'ctx>,
) -> bool {
    match (from_cell_id, to_cell_id) {
        (Some(from_id), Some(to_id)) => {
            if let (Some(from_cell), Some(to_cell)) = (
                haystack_index.get_cell_by_id(from_id as usize),
                haystack_index.get_cell_by_id(to_id as usize),
            ) {
                haystack_index
                    .fanout_set(&from_cell)
                    .map(|fanout| fanout.contains(&to_cell))
                    .unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    // Tests are in svql_query/tests/ directory
}
