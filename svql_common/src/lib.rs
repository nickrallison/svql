//! Common utilities and shared types for the SVQL workspace.
//!
//! This crate provides hardware abstractions, configuration handling, and tool integration
//! used across the SVQL project.
#![allow(clippy::missing_errors_doc)]

pub mod bridge;
mod config;
pub mod design;

pub use crate::bridge::yosys::*;
pub use crate::config::*;
pub use crate::design::*;

pub use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
