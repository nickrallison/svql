# Qwen Code Configuration for scripts

This file provides context for Qwen Code to better understand and assist with the scripts in the svql project.

## Directory Overview

The **scripts** directory contains various shell scripts for building, testing, and developing the svql project.

## Key Scripts

1. **build.sh** - Main build script using CMake
2. **test.sh** - Main test script
3. **dev_shell.sh** - Script to enter development environment (Nix)
4. **svql_driver.ys** - Yosys script for testing the svql driver
5. **test_svql_driver.sh** - Script to test the svql driver
6. **asan_test.sh** - Script for AddressSanitizer testing

## Technologies

- **Shell scripts**: Bash scripting
- **CMake**: Build system for C++ components
- **Cargo**: Build system for Rust components
- **Nix**: Package manager for reproducible builds

## Conventions

- Scripts are written in Bash
- Build scripts use CMake for C++ components
- Test scripts run both Rust and C++ tests
- Development environment scripts use Nix