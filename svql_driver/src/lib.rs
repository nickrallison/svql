pub mod api;
// pub mod cache;
pub mod context;
pub mod driver;
pub mod util;

pub mod prelude {
    pub use crate::driver::Driver;
    pub use svql_subgraph::SubgraphMatch;
}

#[cfg(test)]
mod tests {

    use svql_subgraph::{config::Config, find_subgraphs};

    use crate::driver::{self};

    #[test]
    fn test_otbn_run_time() {
        let driver = driver::Driver::new_workspace().unwrap();

        let haystack_path = "examples/fixtures/larger_designs/json/otbn_core.json";
        let haystack_module = "otbn_core".to_string();

        let haystack_design = driver
            .get(&haystack_path, haystack_module)
            .expect("Failed to read input design");

        let and_path = "examples/patterns/basic/and/verilog/and_gate.v";
        let and_module = "and_gate".to_string();

        let and_design = driver
            .get(&and_path, and_module)
            .expect("Failed to read input design");

        let needle_1_path =
            "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_1_module = "async_en".to_string();
        let needle_design_1 = driver
            .get(&needle_1_path, needle_1_module)
            .expect("Failed to read input design");

        let needle_2_path =
            "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_2_module = "async_mux".to_string();
        let needle_design_2 = driver
            .get(&needle_2_path, needle_2_module)
            .expect("Failed to read input design");

        let needle_3_path =
            "examples/patterns/security/access_control/locked_reg/json/sync_en.json";
        let needle_3_module = "sync_en".to_string();
        let needle_design_3 = driver
            .get(&needle_3_path, needle_3_module)
            .expect("Failed to read input design");

        let needle_4_path =
            "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_4_module = "sync_mux".to_string();
        let needle_design_4 = driver
            .get(&needle_4_path, needle_4_module)
            .expect("Failed to read input design");

        let match_length = false;
        let config = Config::builder().match_length(match_length).none().build();

        let time_start = std::time::Instant::now();

        let results_1 = find_subgraphs(needle_design_1.as_ref(), haystack_design.as_ref(), &config);
        let results_2 = find_subgraphs(needle_design_2.as_ref(), haystack_design.as_ref(), &config);
        let results_3 = find_subgraphs(needle_design_3.as_ref(), haystack_design.as_ref(), &config);
        let results_4 = find_subgraphs(needle_design_4.as_ref(), haystack_design.as_ref(), &config);

        let results_and = find_subgraphs(and_design.as_ref(), haystack_design.as_ref(), &config);

        println!("Found {} matches for needle 1", results_1.matches.len());
        println!("Found {} matches for needle 2", results_2.matches.len());
        println!("Found {} matches for needle 3", results_3.matches.len());
        println!("Found {} matches for needle 4", results_4.matches.len());
        println!("Found {} matches for AND gate", results_and.matches.len());

        let time_elapsed = time_start.elapsed();
        println!("Test completed in {:?}", time_elapsed);

        // test should take less than 2 seconds (0.2 seconds on my machine (AMD 7950X3D) in release and 1.4 seconds in debug)
        // 2000 milis in release or 14000 milis in debug
        let time_expected = if cfg!(debug_assertions) {
            std::time::Duration::from_millis(14000)
        } else {
            std::time::Duration::from_millis(2000)
        };
        assert!(
            time_elapsed < time_expected,
            "Test took too long to run, expected < {:?}, got {:?}",
            time_expected,
            time_elapsed
        );
    }
}
