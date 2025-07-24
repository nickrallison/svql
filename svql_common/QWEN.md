# Qwen Code Configuration for svql_common

This file provides context for Qwen Code to better understand and assist with the svql_common crate.

## Crate Overview

**svql_common** is a Rust crate that provides shared data structures and utilities for the svql project. It's organized as a library with multiple module types (cdylib, staticlib, rlib) to support both Rust and C++ integration.

## Key Modules

1. **config** - Configuration structures and utilities
2. **pat** - Pattern matching related data structures
3. **mat** - Matching result structures

## Key Technologies

- **Rust**: Primary language
- **cxx**: For C++ interop
- **serde/serde_json**: Serialization/deserialization
- **regex**: Regular expression support
- **thiserror**: Error handling

## Dependencies

- cxx = "1.0"
- lazy_static (workspace)
- regex (workspace)
- serde (workspace)
- serde_json (workspace)
- thiserror = "2.0"

## Conventions

- Follow Rust naming conventions
- Use serde for serialization/deserialization
- Error handling with thiserror
- Regular expressions with regex crate
- C++ interop with cxx