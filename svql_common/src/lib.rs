//! Common utilities and shared types for the SVQL workspace.
//!
//! This crate provides configuration handling, build support, and common test cases
//! used across the SVQL project.
#![allow(clippy::missing_errors_doc)]

mod config;
// mod test_cases;
mod yosys;

pub use crate::config::*;
// pub use crate::test_cases::*;
pub use crate::yosys::*;
