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
    use std::path::PathBuf;

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
}
