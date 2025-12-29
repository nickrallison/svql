//! The driver crate manages the lifetimes of hardware designs and their associated indices.
//!
//! It provides a registry to prevent redundant reloading of netlists,
//! manages design loading and caching, and the creation
//! of execution contexts.

pub mod context;
pub mod design_container;
pub mod driver;
pub mod key;

pub use context::Context;
pub use driver::{Driver, DriverError};
pub use key::DriverKey;

pub use prjunnamed_netlist::Design;
