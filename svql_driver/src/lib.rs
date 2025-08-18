

pub mod cache;
pub mod config;
pub mod driver;
pub mod subgraph;
pub mod util;

pub use driver::Driver;
pub use subgraph::SubgraphMatch;

#[cfg(test)]
mod tests {

    use crate::{cache::Cache, subgraph::find_subgraphs, util::load_driver_cached};

    use super::*;

    #[test]
    fn test_otbn_run_time() {
        let mut cache = Cache::new();

        let haystack_path = "examples/larger_designs/otbn_core.json";
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

        let time_start = std::time::Instant::now();

        let results_1 = subgraph::find_subgraphs(
            needle_design_1.design_as_ref(),
            haystack_design.design_as_ref(),
        );
        let results_2 = subgraph::find_subgraphs(
            needle_design_2.design_as_ref(),
            haystack_design.design_as_ref(),
        );
        let results_3 = subgraph::find_subgraphs(
            needle_design_3.design_as_ref(),
            haystack_design.design_as_ref(),
        );
        let results_4 = subgraph::find_subgraphs(
            needle_design_4.design_as_ref(),
            haystack_design.design_as_ref(),
        );

        let results_and =
            subgraph::find_subgraphs(and_design.design_as_ref(), haystack_design.design_as_ref());

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

    #[test]
    fn connectivity_test() {
        let mut cache = Cache::new();

        let haystack_path = "examples/patterns/basic/ff/seq_8_sdffe.v";
        let haystack_driver =
            load_driver_cached(&haystack_path, &mut cache).expect("Failed to read haystack design");

        let needle_path = "examples/patterns/basic/ff/sdffe.v";
        let needle_driver =
            load_driver_cached(&needle_path, &mut cache).expect("Failed to read needle design");

        let search_results = find_subgraphs(
            needle_driver.design_as_ref(),
            haystack_driver.design_as_ref(),
        );

        for m in search_results.iter() {
            assert!(
                m.design_source_of_input_bit("d", 0).is_some(),
                "input d should have a bound design source"
            );
            assert!(
                m.design_driver_of_output_bit("q", 0).is_some(),
                "output q should have a resolved design driver"
            );
        }

        let ms: Vec<_> = search_results.iter().collect();
        let mut matches = 0;
        for m1 in &ms {
            if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
                let dq_net = dq_cell.output()[dq_bit];
                for m2 in &ms {
                    if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                        let sd_net = sd_cell.output()[sd_bit];
                        if dq_net == sd_net {
                            matches += 1;
                        }
                    }
                }
            }
        }

        assert_eq!(
            matches, 7,
            "Expected 7 connections between d and q across matches, found {}",
            matches
        );
    }
}
