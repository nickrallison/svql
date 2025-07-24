# Qwen Code Configuration for svql_pat_lib

This file provides context for Qwen Code to better understand and assist with the svql_pat_lib component.

## Component Overview

**svql_pat_lib** is a C++ component that implements pattern matching functionality for integration with Yosys. It works alongside svql_driver to provide pattern matching capabilities within the Yosys environment.

## Key Files

1. **SvqlPatPass.cpp/hpp** - Main pattern matching pass implementation

## Key Technologies

- **C++**: Primary language
- **Yosys**: Framework for Verilog RTL synthesis
- **CMake**: Build system
- **nlohmann-json**: JSON handling

## Dependencies

- Yosys (installed from source)
- nlohmann-json3-dev
- cmake

## Conventions

- Follow Yosys coding conventions
- C++ files use .cpp extension
- Header files use .hpp extension
- Implementation follows Yosys pass architecture