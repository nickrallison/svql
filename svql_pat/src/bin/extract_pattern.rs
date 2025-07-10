// Example program demonstrating the svql_pat library usage

use svql_pat::{extract_pattern_default, SvqlPatError};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <verilog_file> <module_name>", args[0]);
        eprintln!("Example: {} examples/cwe1234/variant1.v locked_register_example", args[0]);
        std::process::exit(1);
    }
    
    let verilog_file = &args[1];
    let module_name = &args[2];
    
    println!("Extracting pattern from file '{}' for module '{}'...", verilog_file, module_name);
    
    match extract_pattern_default(verilog_file, module_name) {
        Ok(pattern) => {
            println!("\n✅ Successfully extracted pattern:");
            println!("   📁 File: {:?}", pattern.file_loc);
            println!("   📥 Input ports ({}):", pattern.in_ports.len());
            for port in &pattern.in_ports {
                println!("      - {}", port);
            }
            println!("   📤 Output ports ({}):", pattern.out_ports.len());
            for port in &pattern.out_ports {
                println!("      - {}", port);
            }
            if !pattern.inout_ports.is_empty() {
                println!("   🔄 Inout ports ({}):", pattern.inout_ports.len());
                for port in &pattern.inout_ports {
                    println!("      - {}", port);
                }
            }
            
            // Serialize to JSON for easy consumption
            match serde_json::to_string_pretty(&pattern) {
                Ok(json) => {
                    println!("\n📋 JSON representation:");
                    println!("{}", json);
                },
                Err(e) => {
                    eprintln!("⚠️  Warning: Failed to serialize to JSON: {}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            
            // Provide specific help based on error type
            match e {
                SvqlPatError::FileNotFound { .. } => {
                    eprintln!("💡 Tip: Make sure the Verilog file path is correct and the file exists.");
                },
                SvqlPatError::ModuleNotFound { .. } => {
                    eprintln!("💡 Tip: Check that the module name is correct and matches a module in the file.");
                },
                SvqlPatError::SyntaxError { .. } => {
                    eprintln!("💡 Tip: Fix the syntax errors in the Verilog file before extracting patterns.");
                },
                SvqlPatError::YosysExecutionError { .. } => {
                    eprintln!("💡 Tip: Make sure yosys is installed and accessible, and the plugin library is built.");
                },
                _ => {
                    eprintln!("💡 Tip: Check the error message above for more details.");
                }
            }
            
            std::process::exit(1);
        }
    }
}
