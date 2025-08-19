pub mod api;
pub mod cache;
pub mod driver;
pub mod util;

pub mod prelude {
    pub use crate::driver::Driver;
    pub use svql_subgraph::SubgraphMatch;
}

#[cfg(test)]
mod tests {

    use svql_subgraph::{
        config::{self, Config},
        find_subgraphs,
    };

    use crate::{cache::Cache, util::load_driver_cached};

    #[test]
    fn test_otbn_run_time() {
        let mut cache = Cache::new();

        let haystack_path = "examples/fixtures/larger_designs/json/otbn_core.json";
        let haystack_design =
            load_driver_cached(haystack_path, &mut cache).expect("Failed to read input design");

        let and_design = load_driver_cached("examples/patterns/basic/and/and_gate.v", &mut cache)
            .expect("Failed to read input design");

        let needle_path_1 =
            "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_design_1 =
            load_driver_cached(needle_path_1, &mut cache).expect("Failed to read input design");

        let needle_path_2 =
            "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design_2 =
            load_driver_cached(needle_path_2, &mut cache).expect("Failed to read input design");

        let needle_path_3 =
            "examples/patterns/security/access_control/locked_reg/json/sync_en.json";
        let needle_design_3 =
            load_driver_cached(needle_path_3, &mut cache).expect("Failed to read input design");

        let needle_path_4 =
            "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_design_4 =
            load_driver_cached(needle_path_4, &mut cache).expect("Failed to read input design");

        let match_length = false;
        let config = Config::new(match_length);

        let time_start = std::time::Instant::now();

        let results_1 = find_subgraphs(
            needle_design_1.design_as_ref(),
            haystack_design.design_as_ref(),
            &config,
        );
        let results_2 = find_subgraphs(
            needle_design_2.design_as_ref(),
            haystack_design.design_as_ref(),
            &config,
        );
        let results_3 = find_subgraphs(
            needle_design_3.design_as_ref(),
            haystack_design.design_as_ref(),
            &config,
        );
        let results_4 = find_subgraphs(
            needle_design_4.design_as_ref(),
            haystack_design.design_as_ref(),
            &config,
        );

        let results_and = find_subgraphs(
            and_design.design_as_ref(),
            haystack_design.design_as_ref(),
            &config,
        );

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
