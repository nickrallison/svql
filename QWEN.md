# Qwen Code Configuration

This file provides context for Qwen Code to better understand and assist with the svql project.

## Project Overview

**svql** is a Rust-based project for querying and analyzing SystemVerilog code using pattern matching. The project is organized as a Cargo workspace with multiple crates.

## Project Structure

```
svql/
├── svql_common/      # Shared data structures and utilities
├── svql_query/       # Main query engine and CLI
├── svql_pat/         # Pattern matching library (commented out in workspace)
├── svql_driver/      # C++ driver for integration with Yosys
├── svql_pat_lib/     # C++ pattern matching library
├── yosys/            # Fork of Yosys for integration
├── examples/         # Example patterns and test cases
└── scripts/          # Build and test scripts
```

Each directory has its own QWEN.md file with more specific information:
- [svql_common/QWEN.md](svql_common/QWEN.md)
- [svql_query/QWEN.md](svql_query/QWEN.md)
- [svql_pat/QWEN.md](svql_pat/QWEN.md)
- [svql_driver/QWEN.md](svql_driver/QWEN.md)
- [svql_pat_lib/QWEN.md](svql_pat_lib/QWEN.md)
- [examples/QWEN.md](examples/QWEN.md)
- [scripts/QWEN.md](scripts/QWEN.md)

## Key Technologies

- **Rust**: Primary language for pattern matching and query engine
- **C++**: Used for Yosys integration components
- **Yosys**: Open-source framework for Verilog RTL synthesis
- **Cargo**: Rust package manager and build system
- **CMake**: Build system for C++ components
- **Nix**: Package manager for reproducible builds (via flake.nix)

## Dependencies

Key dependencies include:
- cbindgen
- lazy_static
- regex
- serde/serde_json
- thiserror

External dependencies for Yosys:
- bison, flex, libffi, tcl, tk, readline, python3, zlib, pkg-config

## Development Commands

- `./scripts/build.sh` - Build the project
- `./scripts/test.sh` - Run tests
- `./scripts/dev_shell.sh` - Enter development environment (Nix)
- `cargo build` - Build Rust components
- `cargo test` - Test Rust components

## Conventions

- Follow Rust naming conventions
- Use serde for serialization/deserialization
- Error handling with thiserror
- Regular expressions with regex crate
- C++ code follows Yosys conventions

## Workspace Crates

1. **svql_common** - Shared types and utilities
2. **svql_query** - Main query engine implementation
3. **svql_pat** - Pattern matching library (currently disabled in workspace)
4. **svql_driver** - C++ integration with Yosys
5. **svql_pat_lib** - C++ pattern matching library

Note: svql_pat is commented out in the workspace Cargo.toml but still exists in the file system.