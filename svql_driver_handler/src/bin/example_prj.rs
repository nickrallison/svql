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
        // assert_eq!(matches.len(), 207, "Expected 207 matches for needle {}, got {}", needle_name_1, matches.len());

        let matches = subgraph::find_subgraphs(&needle_design_2, &haystack_design);
        

}