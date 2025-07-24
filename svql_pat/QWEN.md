# Qwen Code Configuration for svql_pat

This file provides context for Qwen Code to better understand and assist with the svql_pat crate.

## Crate Overview

**svql_pat** is a Rust crate that implements the pattern matching library for the svql project. It's organized as a library with multiple module types (cdylib, staticlib, rlib) to support both Rust and C++ integration.

Note: This crate is currently commented out in the workspace Cargo.toml but still exists in the file system.

## Key Technologies

- **Rust**: Primary language
- **svql_common**: Shared data structures and utilities
- **serde/serde_json**: Serialization/deserialization
- **regex**: Regular expression support
- **thiserror**: Error handling

## Dependencies

- svql_common = { path = "../svql_common" }
- thiserror (workspace)
- regex (workspace)
- serde (workspace)
- serde_json (workspace)

## Conventions

- Follow Rust naming conventions
- Use serde for serialization/deserialization
- Error handling with thiserror
- Regular expressions with regex crate
- Pattern matching implementation follows standard practices