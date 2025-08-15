use svql_driver_handler::prjunnamed::*;
use svql_driver_handler::prjunnamed::subgraph;

fn main() {

    // env logger
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let haystack_path = "examples/larger_designs/otbn.json";
        // let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
        let haystack_design = read_input(None, haystack_path.to_string()).expect("Failed to read input design");
        let haystack_name = get_name(&haystack_path);

        let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_design = read_input(None, needle_path.to_string()).expect("Failed to read input design");
        let needle_name = get_name(&needle_path);

        // Find subgraphs using the chosen anchor kind
        let matches: Vec<std::collections::HashMap<usize, usize>> = subgraph::find_subgraphs(&needle_design, &haystack_design);
        assert_eq!(matches.len(), 207, "Expected 207 matches for needle {}, against haystack {}, got {}", needle_name, haystack_name, matches.len());

        


}