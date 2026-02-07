//! Yosys integration for SVQL.
//!
//! This module provides types and utilities for interacting with Yosys, including
//! configuration, design paths, and module handling.

mod config;
mod design_path;
mod module;

pub use config::ModuleConfig;
pub use design_path::DesignPath;
pub use module::YosysModule;
