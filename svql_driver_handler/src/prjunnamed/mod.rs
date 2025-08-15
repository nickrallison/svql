use std::{cell, error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;

mod subgraph;

fn read_input(target: Option<Arc<dyn Target>>, name: String) -> Result<prjunnamed_netlist::Design, Box<dyn Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let path = Path::new(&name);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    };

    if name.ends_with(".uir") {
        Ok(prjunnamed_netlist::parse(target, &std::fs::read_to_string(abs_path)?)?)
    } else if name.ends_with(".json") {
        let designs = prjunnamed_yosys_json::import(target, &mut File::open(abs_path)?)?;
        assert_eq!(designs.len(), 1, "can only convert single-module Yosys JSON to Unnamed IR");
        Ok(designs.into_values().next().unwrap())
    } else if name.is_empty() {
        panic!("no input provided")
    } else {
        panic!("don't know what to do with input {name:?}")
    }
}

fn get_name(name: &str) -> String {
    let path = PathBuf::from(name);
    let file_stem = PathBuf::from(path.file_stem().unwrap());
    let file_name = file_stem.file_name().unwrap().to_string_lossy();
    file_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otbn() {
        

        // env logger
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        // let otbn_path = PathBuf::from("examples/larger_designs/otbn.json");
        // let otbn_design = read_input(None, otbn_path.to_string_lossy().to_string()).expect("Failed to read input design");

        let haystack_path = "examples/larger_designs/otbn.json";
        // let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
        let haystack_design = read_input(None, haystack_path.to_string()).expect("Failed to read input design");
        let haystack_name = get_name(&haystack_path);

        let needle_path_1 = "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_design_1 = read_input(None, needle_path_1.to_string()).expect("Failed to read input design");
        let needle_name_1 = get_name(&needle_path_1);

        let needle_path_2 = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design_2 = read_input(None, needle_path_2.to_string()).expect("Failed to read input design");
        let needle_name_2 = get_name(&needle_path_2);

        let needle_path_3 = "examples/patterns/security/access_control/locked_reg/json/sync_en.json";
        let needle_design_3 = read_input(None, needle_path_3.to_string()).expect("Failed to read input design");
        let needle_name_3 = get_name(&needle_path_3);

        let needle_path_4 = "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_design_4 = read_input(None, needle_path_4.to_string()).expect("Failed to read input design");
        let needle_name_4 = get_name(&needle_path_4);

        // Find subgraphs using the chosen anchor kind
        let matches: Vec<std::collections::HashMap<usize, usize>> = subgraph::find_subgraphs(&needle_design_1, &haystack_design);
        assert_eq!(matches.len(), 207, "Expected 207 matches for needle {}, got {}", needle_name_1, matches.len());

        let matches = subgraph::find_subgraphs(&needle_design_2, &haystack_design);
        assert_eq!(matches.len(), 0, "Expected 0 matches for needle {}, got {}", needle_name_2, matches.len());

        let matches = subgraph::find_subgraphs(&needle_design_3, &haystack_design);
        assert_eq!(matches.len(), 207, "Expected 207 matches for needle {}, got {}", needle_name_3, matches.len());

        let matches = subgraph::find_subgraphs(&needle_design_4, &haystack_design);
        assert_eq!(matches.len(), 0, "Expected 0 matches for needle {}, got {}", needle_name_4, matches.len());

    }
}