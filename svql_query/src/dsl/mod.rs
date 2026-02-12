//! The Pattern Definition Language (DSL).
//!
//! This module provides traits and utilities for defining query patterns in SVQL.

/// Path navigation and port selection logic.
pub mod selector;
pub mod traits;

pub use selector::*;
pub use traits::*;
