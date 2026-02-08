//! Pre-defined patterns and hardware primitives for SVQL queries.
//!
//! This library provides:
//! - **Primitives**: Basic combinational and sequential logic gates (AND, OR, DFF, etc.)
//! - **Security**: Patterns for detecting hardware security vulnerabilities (CWE patterns)
//! - **Experimental**: Emerging patterns for testing and development
//!
//! Each module contains ready-to-use pattern implementations that can be
//! composed or extended for custom queries.

pub mod experimental;
pub mod primitives;
pub mod security;

pub use primitives::*;
