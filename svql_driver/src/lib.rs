pub mod api;
pub mod driver;
pub mod util;

pub mod prelude {
    pub use crate::driver::DesignKey;
    pub use crate::driver::Driver;
    pub use svql_subgraph::SubgraphMatch;
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use svql_subgraph::{config::Config, find_subgraphs};

    use crate::util::new_shared_driver;

    #[test]
    fn test_otbn_run_time() {
        let driver = new_shared_driver().expect("driver");

        let config = Config::builder().match_length(false).none().build();

        // Hold Arcs so the borrows into results live long enough
        let hay_arc = driver
            .get_by_path(
                &PathBuf::from("examples/fixtures/larger_designs/json/otbn_core.json"),
                "otbn_core",
            )
            .unwrap();

        let and_arc = driver
            .get_by_path(
                &PathBuf::from("examples/patterns/basic/and/verilog/and_gate.v"),
                "and_gate",
            )
            .unwrap();

        let n1_arc = driver
            .get_by_path(
                &PathBuf::from(
                    "examples/patterns/security/access_control/locked_reg/json/async_en.json",
                ),
                "async_en",
            )
            .unwrap();
        let n2_arc = driver
            .get_by_path(
                &PathBuf::from(
                    "examples/patterns/security/access_control/locked_reg/json/async_mux.json",
                ),
                "async_mux",
            )
            .unwrap();
        let n3_arc = driver
            .get_by_path(
                &PathBuf::from(
                    "examples/patterns/security/access_control/locked_reg/json/sync_en.json",
                ),
                "sync_en",
            )
            .unwrap();
        let n4_arc = driver
            .get_by_path(
                &PathBuf::from(
                    "examples/patterns/security/access_control/locked_reg/json/sync_mux.json",
                ),
                "sync_mux",
            )
            .unwrap();

        let time_start = std::time::Instant::now();

        let results_1 = find_subgraphs(n1_arc.as_ref(), hay_arc.as_ref(), &config);
        let results_2 = find_subgraphs(n2_arc.as_ref(), hay_arc.as_ref(), &config);
        let results_3 = find_subgraphs(n3_arc.as_ref(), hay_arc.as_ref(), &config);
        let results_4 = find_subgraphs(n4_arc.as_ref(), hay_arc.as_ref(), &config);

        let results_and = find_subgraphs(and_arc.as_ref(), hay_arc.as_ref(), &config);

        println!("Found {} matches for needle 1", results_1.matches.len());
        println!("Found {} matches for needle 2", results_2.matches.len());
        println!("Found {} matches for needle 3", results_3.matches.len());
        println!("Found {} matches for needle 4", results_4.matches.len());
        println!("Found {} matches for AND gate", results_and.matches.len());

        let time_elapsed = time_start.elapsed();
        println!("Test completed in {:?}", time_elapsed);

        // test should take less than 2 seconds in release or 14 seconds in debug
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
    fn test_port_matches() {
        let driver = new_shared_driver().expect("Failed to create driver");

        // Hold Arcs in locals so borrows outlive the search_results
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

        let pat_path = workspace.join("examples/patterns/basic/ff/verilog/sdffe.v");
        let pat_arc = driver
            .get_by_path(&pat_path, "sdffe")
            .expect("pattern design present");
        let hay_path = workspace.join("examples/fixtures/basic/ff/verilog/seq_sdffe.v");
        let hay_arc = driver
            .get_by_path(&hay_path, "seq_sdffe")
            .expect("haystack design present");

        let config = Config::builder().exact_length().none().build();

        let search_results = find_subgraphs(pat_arc.as_ref(), hay_arc.as_ref(), &config);

        // Every match should resolve both d (input) and q (output) via O(1) helpers
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

        // There should exist a pair of matches where q of one drives d of the other.
        let ms: Vec<_> = search_results.iter().collect();
        let mut found_port_matches = 0;
        for m1 in ms.iter() {
            if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
                let valid_dq_cell = dq_cell
                    .try_into_valid_cell_wrapper(hay_arc.as_ref())
                    .expect("valid cell wrapper");
                let dq_net = valid_dq_cell.output()[dq_bit];

                for m2 in ms.iter() {
                    if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                        let valid_sd_cell = sd_cell
                            .try_into_valid_cell_wrapper(hay_arc.as_ref())
                            .expect("valid cell wrapper");
                        let sd_net = valid_sd_cell.output()[sd_bit];
                        if dq_net == sd_net {
                            println!("Found connection at cell: {:#?}", valid_dq_cell.metadata());
                            found_port_matches += 1;
                        }
                    }
                }
            }
        }

        assert_eq!(found_port_matches, 7);
    }
}
