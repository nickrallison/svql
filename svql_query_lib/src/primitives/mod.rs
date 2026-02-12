//! Primitive hardware components.
//!
//! This module provides pre-defined primitive gates and flip-flops that
//! match directly against cell types in hardware designs.

pub mod dff;
pub mod gates;
/// Recursive pattern implementations for tree structures.
pub mod rec;

// Re-export commonly used primitives
pub use dff::*;
pub use gates::*;
pub use rec::*;
