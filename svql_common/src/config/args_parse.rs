use std::ffi::{CStr};
use std::os::raw::c_char;
use crate::config::{SvqlRuntimeConfig, CSvqlRuntimeConfig};

/// Helper: Convert C argv to Vec<String>
fn argv_to_vec(argc: i32, argv: *const *const c_char) -> Vec<String> {
    if argv.is_null() || argc <= 0 {
        return Vec::new();
    }
    unsafe {
        (0..argc)
            .map(|i| {
                let ptr = *argv.offset(i as isize);
                if ptr.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            })
            .collect()
    }
}

/// The main logic: parse args and build config
pub fn svql_runtime_config_from_args_logic(args: &[String]) -> SvqlRuntimeConfig {
    let mut cfg = SvqlRuntimeConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-pat" if i + 2 < args.len() => {
                let pat_filename = args[i + 1].clone();
                let pat_module_name = args[i + 2].clone();

                cfg.pat_filename = pat_filename;
                
                if !pat_module_name.starts_with('\\') {
                    cfg.pat_module_name = format!("\\{}", pat_module_name);
                } else {
                    cfg.pat_module_name = pat_module_name;
                }
                i += 3;
            }
            "-verbose" => {
                cfg.verbose = true;
                i += 1;
            }
            "-constports" => {
                cfg.const_ports = true;
                i += 1;
            }
            "-nodefaultswaps" => {
                cfg.nodefaultswaps = true;
                i += 1;
            }
            "-compat" if i + 2 < args.len() => {
                cfg.compat_pairs.push((args[i + 1].clone(), args[i + 2].clone()));
                i += 3;
            }
            "-swap" if i + 2 < args.len() => {
                let type_name = args[i + 1].clone();
                let ports: Vec<String> = args[i + 2]
                    .split(|c| c == ',' || c == '\t' || c == '\r' || c == '\n' || c == ' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                cfg.swap_ports.push((type_name, ports));
                i += 3;
            }
            "-perm" if i + 3 < args.len() => {
                let type_name = args[i + 1].clone();
                let left: Vec<String> = args[i + 2]
                    .split(|c| c == ',' || c == '\t' || c == '\r' || c == '\n' || c == ' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                let right: Vec<String> = args[i + 3]
                    .split(|c| c == ',' || c == '\t' || c == '\r' || c == '\n' || c == ' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                cfg.perm_ports.push((type_name, left, right));
                i += 4;
            }
            "-cell_attr" if i + 1 < args.len() => {
                cfg.cell_attr.push(args[i + 1].clone());
                i += 2;
            }
            "-wire_attr" if i + 1 < args.len() => {
                cfg.wire_attr.push(args[i + 1].clone());
                i += 2;
            }
            "-ignore_parameters" => {
                cfg.ignore_parameters = true;
                i += 1;
            }
            "-ignore_param" if i + 2 < args.len() => {
                cfg.ignore_param.push((args[i + 1].clone(), args[i + 2].clone()));
                i += 3;
            }
            _ => {
                // Unknown or incomplete option, break or skip
                i += 1;
            }
        }
    }

    cfg
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_from_args(argc: i32, argv: *const *const c_char) -> CSvqlRuntimeConfig {
    let args = argv_to_vec(argc, argv);
    let rust_cfg = svql_runtime_config_from_args_logic(&args);
    CSvqlRuntimeConfig::from(&rust_cfg)
}