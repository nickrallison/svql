//! Integration test suite for the SVQL pattern library.
//!
//! Validates CWE detectors and primitive logic against verified
//! hardware design fixtures.

mod basic;
mod cwe;
mod primitives;

#[macro_use]
mod test_harness;
