//! Common utilities and shared types for the SVQL workspace.
//!
//! This crate provides hardware abstractions, configuration handling, and tool integration
//! used across the SVQL project.
#![allow(clippy::missing_errors_doc)]

pub mod bridge;
pub mod design;
mod config;

pub use crate::config::*;
pub use crate::design::*;
pub use crate::bridge::yosys::*;

pub use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
