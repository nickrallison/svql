use svql_driver::{
    prelude::DesignKey,
    util::{ensure_loaded, new_shared_driver},
};
use svql_subgraph::{config::Config, find_subgraphs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let driver = new_shared_driver()?;

    let hay_key: DesignKey =
        ensure_loaded(&driver, "examples/fixtures/basic/ff/verilog/seq_sdffe.v")?;
    let needle_key: DesignKey =
        ensure_loaded(&driver, "examples/patterns/basic/ff/verilog/sdffe.v")?;

    let config = Config::builder().exact_length().none().build();

    let search_results = find_subgraphs(
        driver.get(&needle_key).unwrap().as_ref(),
        driver.get(&hay_key).unwrap().as_ref(),
        &config,
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
    let mut found = false;
    for m1 in &ms {
        if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
            let dq_net = dq_cell.output()[dq_bit];
            for m2 in &ms {
                if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                    let sd_net = sd_cell.output()[sd_bit];
                    if dq_net == sd_net {
                        println!("Found connection at cell: {:#?}", dq_cell.metadata());
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
