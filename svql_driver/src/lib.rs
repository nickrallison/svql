//! Design driver for loading, caching, and managing hardware designs.
//!
//! The driver crate provides:
//! - `Driver`: Central manager for loading and caching designs with graph indices
//! - `DriverKey`: Unique identifier for designs (path + module name)
//! - `DesignContainer`: Self-referencing container pairing designs with graph indices
//!
//! The driver prevents redundant reloading of designs by caching them keyed by
//! (file path, module name). This enables efficient multi-query execution on the same designs.

pub mod design_container;
pub mod driver;
pub mod key;

pub use driver::{Driver, DriverError};
pub use key::DriverKey;

pub use prjunnamed_netlist::Design;
