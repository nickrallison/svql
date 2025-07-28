// Advanced usage integration tests for svql_pat library
// Tests the extract_pattern function with custom paths

use svql_pat::{SvqlPatError, extract_pattern};

#[test]
fn test_extract_pattern_with_custom_paths() {
    // Test extract_pattern function with custom yosys and plugin paths
    // This demonstrates the advanced usage that was shown in the README

    // Try to find yosys and plugin in the workspace
    let yosys_path = std::path::PathBuf::from("yosys/yosys");
    let plugin_path = std::path::PathBuf::from("build/svql_pat_lib/libsvql_pat_lib.so");

    // Only run this test if the dependencies exist
    if yosys_path.exists() && plugin_path.exists() {
        let result = extract_pattern(
            "examples/cwe1234/variant1.v",
            "locked_register_example",
            Some(yosys_path.to_str().unwrap()),
            Some(plugin_path.to_str().unwrap()),
        );

        match result {
            Ok(pattern) => {
                assert_eq!(pattern.file_loc, "examples/cwe1234/variant1.v");
                assert!(!pattern.in_ports.is_empty());
                assert!(!pattern.out_ports.is_empty());

                println!("Advanced usage test successful:");
                println!("  Used custom yosys path: {:?}", yosys_path);
                println!("  Used custom plugin path: {:?}", plugin_path);
                println!(
                    "  Extracted {} input ports, {} output ports",
                    pattern.in_ports.len(),
                    pattern.out_ports.len()
                );
            }
            Err(e) => {
                panic!("Failed with custom paths: {}", e);
            }
        }
    } else {
        println!("Skipping custom paths test - dependencies not found");
        println!("  yosys exists: {}", yosys_path.exists());
        println!("  plugin exists: {}", plugin_path.exists());
    }
}

#[test]
fn test_extract_pattern_with_invalid_paths() {
    // Test that extract_pattern fails gracefully with invalid paths

    let result = extract_pattern(
        "examples/cwe1234/variant1.v",
        "locked_register_example",
        Some("/invalid/yosys/path"),
        Some("/invalid/plugin/path"),
    );

    match result {
        Ok(_) => panic!("Should have failed with invalid paths"),
        Err(SvqlPatError::YosysExecutionError { details }) => {
            assert!(details.contains("Failed to run yosys") || details.contains("yosys"));
            println!("Correctly failed with invalid yosys path: {}", details);
        }
        Err(e) => {
            println!("Failed with different error (also acceptable): {}", e);
        }
    }
}

#[test]
fn test_library_api_consistency() {
    // Test that both extract_pattern and extract_pattern_default produce the same results
    // when using default paths

    let file = "examples/cwe1234/variant1.v";
    let module = "locked_register_example";

    let result1 = svql_pat::extract_pattern_default(file, module);
    let result2 = extract_pattern(file, module, None::<&str>, None::<&str>);

    match (result1, result2) {
        (Ok(pattern1), Ok(pattern2)) => {
            assert_eq!(pattern1.file_loc, pattern2.file_loc);
            assert_eq!(pattern1.in_ports, pattern2.in_ports);
            assert_eq!(pattern1.out_ports, pattern2.out_ports);
            assert_eq!(pattern1.inout_ports, pattern2.inout_ports);
            println!("Both functions produced identical results");
        }
        (Err(e1), Err(e2)) => {
            // Both functions should fail in the same way
            println!("Both functions failed consistently:");
            println!("  extract_pattern_default: {}", e1);
            println!("  extract_pattern: {}", e2);
        }
        _ => {
            panic!("Functions produced inconsistent results");
        }
    }
}
