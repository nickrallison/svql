//! Common utilities and shared types for the SVQL workspace.
//!
//! This crate provides configuration handling, build support, and common test cases
//! used across the SVQL project.
#![allow(clippy::missing_errors_doc)]

mod config;
pub mod prelude;
mod yosys;

pub use crate::config::*;
pub use crate::yosys::*;
pub use crate::prelude::{HashMap, HashSet};
