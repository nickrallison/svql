//! Traits defining the behavior of SVQL queries.
//!
//! This module contains the core traits that all query types (netlists, composites,
//! enum composites) must implement to be used in the SVQL system.

pub mod netlist;
pub mod composite;
pub mod variant;
