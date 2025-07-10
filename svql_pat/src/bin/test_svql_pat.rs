// Test program for svql_pat library

use svql_pat::{extract_pattern_default, SvqlPatError};

fn main() {
    println!("Testing svql_pat library...");
    
    // Test with the known working example
    match extract_pattern_default("examples/cwe1234/variant1.v", "locked_register_example") {
        Ok(pattern) => {
            println!("Successfully extracted pattern:");
            println!("  File: {:?}", pattern.file_loc);
            println!("  Input ports: {:?}", pattern.in_ports);
            println!("  Output ports: {:?}", pattern.out_ports);
            println!("  Inout ports: {:?}", pattern.inout_ports);
        },
        Err(e) => {
            println!("Error extracting pattern: {}", e);
            std::process::exit(1);
        }
    }
    
    // Test with non-existent file
    println!("\nTesting with non-existent file...");
    match extract_pattern_default("non_existent.v", "some_module") {
        Ok(_) => {
            println!("ERROR: Should have failed with non-existent file");
            std::process::exit(1);
        },
        Err(SvqlPatError::FileNotFound { path }) => {
            println!("Correctly detected non-existent file: {:?}", path);
        },
        Err(e) => {
            println!("Unexpected error: {}", e);
            std::process::exit(1);
        }
    }
    
    // Test with non-existent module
    println!("\nTesting with non-existent module...");
    match extract_pattern_default("examples/cwe1234/variant1.v", "non_existent_module") {
        Ok(_) => {
            println!("ERROR: Should have failed with non-existent module");
            std::process::exit(1);
        },
        Err(SvqlPatError::ModuleNotFound { module, file }) => {
            println!("Correctly detected non-existent module '{}' in file {:?}", module, file);
        },
        Err(e) => {
            println!("Unexpected error: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\nAll tests passed!");
}
