// Integration tests for svql_pat library

use std::path::PathBuf;
use svql_pat::{SvqlPatError, extract_pattern_default};

#[test]
fn test_extract_pattern_success() {
    // Test with the known working example
    let result =
        extract_pattern_default("../examples/cwe1234/variant1.v", "locked_register_example");

    match result {
        Ok(pattern) => {
            // Verify the pattern structure
            assert_eq!(pattern.file_loc, "examples/cwe1234/variant1.v");
            assert!(!pattern.in_ports.is_empty(), "Should have input ports");
            assert!(!pattern.out_ports.is_empty(), "Should have output ports");

            // Check for expected ports (from the original CLI output)
            assert!(pattern.in_ports.contains(&"\\data_in".to_string()));
            assert!(pattern.in_ports.contains(&"\\clk".to_string()));
            assert!(pattern.in_ports.contains(&"\\resetn".to_string()));
            assert!(pattern.out_ports.contains(&"\\data_out".to_string()));

            println!("Successfully extracted pattern:");
            println!("  File: {:?}", pattern.file_loc);
            println!(
                "  Input ports ({}): {:?}",
                pattern.in_ports.len(),
                pattern.in_ports
            );
            println!(
                "  Output ports ({}): {:?}",
                pattern.out_ports.len(),
                pattern.out_ports
            );
            println!(
                "  Inout ports ({}): {:?}",
                pattern.inout_ports.len(),
                pattern.inout_ports
            );
        }
        Err(e) => {
            panic!("Failed to extract pattern: {}", e);
        }
    }
}

#[test]
fn test_file_not_found_error() {
    // Test with non-existent file
    let result = extract_pattern_default("non_existent.v", "some_module");

    match result {
        Ok(_) => {
            panic!("Should have failed with non-existent file");
        }
        Err(SvqlPatError::FileNotFound { path }) => {
            assert_eq!(path, PathBuf::from("non_existent.v"));
            println!("Correctly detected non-existent file: {:?}", path);
        }
        Err(e) => {
            panic!("Unexpected error: {}", e);
        }
    }
}

#[test]
fn test_module_not_found_error() {
    // Test with non-existent module
    let result = extract_pattern_default("examples/cwe1234/variant1.v", "non_existent_module");

    match result {
        Ok(_) => {
            panic!("Should have failed with non-existent module");
        }
        Err(SvqlPatError::ModuleNotFound { module, file }) => {
            assert_eq!(module, "non_existent_module");
            assert_eq!(file, PathBuf::from("examples/cwe1234/variant1.v"));
            println!(
                "Correctly detected non-existent module '{}' in file {:?}",
                module, file
            );
        }
        Err(e) => {
            panic!("Unexpected error: {}", e);
        }
    }
}

#[test]
fn test_json_serialization() {
    // Test that extracted patterns can be serialized to JSON
    let result = extract_pattern_default("examples/cwe1234/variant1.v", "locked_register_example");

    match result {
        Ok(pattern) => {
            // Test JSON serialization
            let json_result = serde_json::to_string_pretty(&pattern);
            assert!(json_result.is_ok(), "Failed to serialize pattern to JSON");

            let json = json_result.unwrap();
            assert!(json.contains("file_loc"));
            assert!(json.contains("in_ports"));
            assert!(json.contains("out_ports"));
            assert!(json.contains("inout_ports"));

            println!("JSON representation:");
            println!("{}", json);
        }
        Err(e) => {
            panic!("Failed to extract pattern for JSON test: {}", e);
        }
    }
}

#[test]
fn test_error_handling_comprehensive() {
    // Test various error conditions and their messages

    // File not found
    match extract_pattern_default("does_not_exist.v", "module") {
        Err(SvqlPatError::FileNotFound { .. }) => {
            println!("✓ FileNotFound error handled correctly");
        }
        _ => panic!("Expected FileNotFound error"),
    }

    // Module not found (using existing file)
    match extract_pattern_default("examples/cwe1234/variant1.v", "invalid_module") {
        Err(SvqlPatError::ModuleNotFound { .. }) => {
            println!("✓ ModuleNotFound error handled correctly");
        }
        _ => panic!("Expected ModuleNotFound error"),
    }
}

/// Helper function to demonstrate library usage patterns
/// This is similar to what was in the extract_pattern.rs bin file
#[test]
fn test_library_usage_examples() {
    // Example 1: Basic usage
    let result = extract_pattern_default("examples/cwe1234/variant1.v", "locked_register_example");

    if let Ok(pattern) = result {
        // This demonstrates how users would typically use the library
        assert!(!pattern.in_ports.is_empty());
        assert!(!pattern.out_ports.is_empty());

        // Show how to access pattern data
        for port in &pattern.in_ports {
            println!("Input port: {}", port);
        }

        for port in &pattern.out_ports {
            println!("Output port: {}", port);
        }

        // Show JSON serialization usage
        if let Ok(json) = serde_json::to_string_pretty(&pattern) {
            assert!(json.len() > 0);
        }
    }
}

#[test]
fn test_error_messages_are_helpful() {
    // Test that error messages provide helpful information

    // Test file not found error message
    if let Err(e) = extract_pattern_default("missing.v", "module") {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("File not found"));
        assert!(error_msg.contains("missing.v"));
    }

    // Test module not found error message
    if let Err(e) = extract_pattern_default("examples/cwe1234/variant1.v", "missing_module") {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("Module"));
        assert!(error_msg.contains("missing_module"));
        assert!(error_msg.contains("not found"));
    }
}
