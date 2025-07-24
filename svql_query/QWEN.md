# Qwen Code Configuration for svql_query

This file provides context for Qwen Code to better understand and assist with the svql_query crate.

## Crate Overview

**svql_query** is the main query engine and CLI interface for the svql project. It uses the shared components from svql_common to provide a command-line tool for querying SystemVerilog code.

## Key Modules

1. **main** - Entry point and CLI interface
2. **module** - Module representation and querying logic
3. **ports** - Port handling functionality
4. **query** - Query execution logic
5. **driver** - Communication with Yosys driver
6. **examples** - Example query implementations

## Key Technologies

- **Rust**: Primary language
- **svql_common**: Shared data structures and utilities
- **serde/serde_json**: Serialization/deserialization
- **thiserror**: Error handling
- **log**: Logging

## Dependencies

- svql_common = { path = "../svql_common" }
- serde = { version = "1.0", features = ["derive"] }
- serde_json = "1.0"
- thiserror = "2.0"
- lazy_static (workspace)
- log = "0.4.27"

## Conventions

- Follow Rust naming conventions
- Use serde for serialization/deserialization
- Error handling with thiserror
- Logging with the log crate
- Examples in the examples module demonstrate usage patterns