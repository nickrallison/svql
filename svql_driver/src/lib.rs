//! The main driver for SVQL execution.
//!
//! This crate manages the execution context, design containers, and the overall
//! driver logic for processing SVQL queries.

pub mod context;
pub mod design_container;
pub mod driver;
pub mod key;

pub use context::Context;
pub use driver::{Driver, DriverError};
pub use key::DriverKey;

pub use prjunnamed_netlist::Design;
