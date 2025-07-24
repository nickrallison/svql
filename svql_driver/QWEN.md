# Qwen Code Configuration for svql_driver

This file provides context for Qwen Code to better understand and assist with the svql_driver component.

## Component Overview

**svql_driver** is a C++ component that acts as a bridge between svql and Yosys. It implements a Yosys pass that can be used to analyze and query SystemVerilog designs.

## Key Files

1. **SvqlPass.cpp/hpp** - Main Yosys pass implementation
2. **GraphConversion.cpp/hpp** - Graph conversion utilities
3. **SubCircuitReSolver.cpp/hpp** - Subcircuit resolution logic

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