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

## Testing

Run the test suite to verify the library functionality:

```bash
cargo test --package svql_pat
```

The test suite includes:
- Integration tests demonstrating basic usage
- Error handling tests for various failure modes
- Advanced usage tests with custom paths
- JSON serialization tests

## Usage Examples

### Example 1: Extract and Display Pattern Information

```rust
use svql_pat::extract_pattern_default;

fn main() {
    match extract_pattern_default("examples/cwe1234/variant1.v", "locked_register_example") {
        Ok(pattern) => {
            println!("Successfully extracted pattern:");
            println!("  File: {:?}", pattern.file_loc);
            println!("  Input ports ({}): {:?}", pattern.in_ports.len(), pattern.in_ports);
            println!("  Output ports ({}): {:?}", pattern.out_ports.len(), pattern.out_ports);
            if !pattern.inout_ports.is_empty() {
                println!("  Inout ports ({}): {:?}", pattern.inout_ports.len(), pattern.inout_ports);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Example 2: JSON Export

```rust
use svql_pat::extract_pattern_default;

fn export_pattern_to_json(verilog_file: &str, module_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pattern = extract_pattern_default(verilog_file, module_name)?;
    let json = serde_json::to_string_pretty(&pattern)?;
    Ok(json)
}
```

### Example 3: Batch Processing

```rust
use svql_pat::{extract_pattern_default, SvqlPatError};

fn process_multiple_modules(files_and_modules: &[(&str, &str)]) {
    for (file, module) in files_and_modules {
        match extract_pattern_default(file, module) {
            Ok(pattern) => {
                println!("✓ {} -> {} ports total", 
                    module, 
                    pattern.in_ports.len() + pattern.out_ports.len() + pattern.inout_ports.len()
                );
            }
            Err(SvqlPatError::FileNotFound { .. }) => {
                eprintln!("✗ File not found: {}", file);
            }
            Err(SvqlPatError::ModuleNotFound { .. }) => {
                eprintln!("✗ Module '{}' not found in {}", module, file);
            }
            Err(e) => {
                eprintln!("✗ Error processing {}: {}", file, e);
            }
        }
    }
}
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
2. Verify the library compiles without warnings: `cargo check`
3. Test the example usage in the integration tests

## License

This project follows the same license as the parent SVQL project.
