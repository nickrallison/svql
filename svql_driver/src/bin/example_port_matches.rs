use std::path::Path;

use svql_driver::util::new_shared_driver;
use svql_subgraph::{config::Config, find_subgraphs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let driver = new_shared_driver()?;

    // Hold Arcs in locals so borrows outlive the search_results
    let pat_arc = driver
        .get_by_path(
            Path::new("examples/patterns/basic/ff/verilog/sdffe.v"),
            "sdffe",
        )
        .expect("pattern design present");
    let hay_arc = driver
        .get_by_path(
            Path::new("examples/fixtures/basic/ff/verilog/seq_sdffe.v"),
            "seq_sdffe",
        )
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
    let mut found = false;
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
                        found = true;
                    }
                }
            }
        }
    }
    if !found {
        println!("No connections found between matches.");
    }

    Ok(())
}
