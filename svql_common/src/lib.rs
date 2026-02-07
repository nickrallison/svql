//! Common utilities and shared types for the SVQL workspace.
//!
//! This crate provides configuration handling, build support, and common test cases
//! used across the SVQL project.
#![allow(clippy::missing_errors_doc)]

pub mod cell;
mod config;
pub mod wire;
mod yosys;

pub use crate::cell::*;
pub use crate::config::*;
pub use crate::wire::*;
pub use crate::yosys::*;

pub use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
