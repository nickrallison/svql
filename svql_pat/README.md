# SVQL Pattern Extraction Library

This library provides a Rust interface for extracting interface patterns from Verilog modules using the yosys synthesis tool with the `svql_pat_lib` plugin.

## Overview

The `svql_pat` library allows you to programmatically analyze Verilog files and extract structured information about module interfaces, including input ports, output ports, and inout ports. It leverages the yosys synthesis framework and a custom plugin (`svql_pat_lib`) to parse Verilog code and extract interface patterns.

## Features

- **Pattern Extraction**: Extract detailed interface information from Verilog modules
- **Error Handling**: Comprehensive error types with helpful diagnostic messages
- **JSON Serialization**: Serialize extracted patterns to JSON for easy integration
- **Flexible Configuration**: Support for custom yosys and plugin paths
- **Safety**: Robust error handling for common failure modes (file not found, module not found, syntax errors)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
svql_pat = { path = "path/to/svql_pat" }
```

## Prerequisites

Before using this library, ensure you have:

1. **Yosys**: The yosys synthesis tool must be installed and accessible
2. **Plugin**: The `svql_pat_lib.so` plugin must be built and available
3. **Build System**: The project uses CMake to build the C++ components

## Quick Start

### Basic Usage

```rust
use svql_pat::extract_pattern_default;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Extract pattern from a Verilog file
    let pattern = extract_pattern_default("examples/cwe1234/variant1.v", "locked_register_example")?;
    
    println!("Module: {:?}", pattern.file_loc);
    println!("Input ports: {:?}", pattern.in_ports);
    println!("Output ports: {:?}", pattern.out_ports);
    println!("Inout ports: {:?}", pattern.inout_ports);
    
    Ok(())
}
```

### Advanced Usage with Custom Paths

```rust
use svql_pat::extract_pattern;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pattern = extract_pattern(
        "path/to/file.v",
        "module_name",
        Some("/custom/path/to/yosys"),
        Some("/custom/path/to/libsvql_pat_lib.so")
    )?;
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&pattern)?;
    println!("{}", json);
    
    Ok(())
}
```

## Error Handling

The library provides detailed error types to help diagnose issues:

```rust
use svql_pat::{extract_pattern_default, SvqlPatError};

match extract_pattern_default("file.v", "module") {
    Ok(pattern) => {
        // Handle successful extraction
        println!("Pattern extracted successfully!");
    },
    Err(SvqlPatError::FileNotFound { path }) => {
        eprintln!("File not found: {:?}", path);
    },
    Err(SvqlPatError::ModuleNotFound { module, file }) => {
        eprintln!("Module '{}' not found in {:?}", module, file);
    },
    Err(SvqlPatError::SyntaxError { file, details }) => {
        eprintln!("Syntax error in {:?}: {}", file, details);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## CLI Tool

The library includes a command-line tool for quick pattern extraction:

```bash
# Build the project
cmake -B build && cmake --build build --parallel 4

# Run the CLI tool
cargo run --package svql_pat --bin extract_pattern examples/cwe1234/variant1.v locked_register_example
```

Example output:
```
✅ Successfully extracted pattern:
   📁 File: "examples/cwe1234/variant1.v"
   📥 Input ports (7):
      - \data_in
      - \clk
      - \resetn
      - \write
      - \lock
      - \scan_mode
      - \debug_unlocked
   📤 Output ports (1):
      - \data_out

📋 JSON representation:
{
  "file_loc": "examples/cwe1234/variant1.v",
  "in_ports": [
    "\\data_in",
    "\\clk",
    "\\resetn",
    "\\write",
    "\\lock",
    "\\scan_mode",
    "\\debug_unlocked"
  ],
  "out_ports": [
    "\\data_out"
  ],
  "inout_ports": []
}
```

## Testing

Run the test suite:

```bash
cargo test --package svql_pat
```

Or run the integration test:

```bash
cargo run --package svql_pat --bin test_svql_pat
```

## Architecture

The library works by:

1. **Invoking Yosys**: Calls the yosys synthesis tool with the custom `svql_pat_lib` plugin
2. **Parsing Output**: Uses regex patterns to extract JSON-formatted pattern data from yosys logs
3. **Error Detection**: Analyzes yosys output for error conditions and provides meaningful error messages
4. **Deserialization**: Converts the extracted JSON into Rust `Pattern` structs

## API Reference

### Main Functions

- `extract_pattern()`: Extract pattern with custom paths
- `extract_pattern_default()`: Extract pattern using default paths

### Error Types

- `SvqlPatError::FileNotFound`: Input file doesn't exist
- `SvqlPatError::ModuleNotFound`: Module not found in file
- `SvqlPatError::SyntaxError`: Verilog syntax errors
- `SvqlPatError::YosysExecutionError`: Problems running yosys
- `SvqlPatError::ParseError`: Issues parsing yosys output
- `SvqlPatError::JsonError`: JSON parsing errors
- `SvqlPatError::PatternCreationError`: Pattern creation failures

### Data Types

The `Pattern` struct (from `svql_common`) contains:
- `file_loc: PathBuf`: Path to the source file
- `in_ports: Vec<String>`: List of input port names
- `out_ports: Vec<String>`: List of output port names  
- `inout_ports: Vec<String>`: List of inout port names

## Contributing

1. Ensure all tests pass: `cargo test`
2. Run the example: `cargo run --package svql_pat --bin extract_pattern examples/cwe1234/variant1.v locked_register_example`
3. Check for compilation warnings: `cargo check`

## License

This project follows the same license as the parent SVQL project.
