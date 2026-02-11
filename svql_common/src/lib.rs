//! Shared types, configuration, and tool integrations for SVQL.
//!
//! This crate provides core abstractions used across the SVQL project:
//! - **Design**: Hardware cell types, wire references, and source location tracking
//! - **Config**: Query configuration (match strategies, Yosys parameters, parallelism)
//! - **Bridge**: Integration with Yosys for design processing and transformation
//!
//! The types defined here form the bridge between high-level pattern queries
//! and low-level netlist data structures.

pub mod bridge;
mod config;
pub mod design;
pub mod index;

pub use crate::bridge::yosys::*;
pub use crate::config::*;
pub use crate::design::*;
pub use crate::index::*;

pub use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
