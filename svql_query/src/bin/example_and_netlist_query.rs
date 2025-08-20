use svql_driver::prelude::{DesignKey, Driver};
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::netlist::ctx_for;
use svql_query::queries::netlist::basic::and::and_gate::AndGate;
use svql_subgraph::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // One shared driver for everything.
    let driver = Driver::new()?;

    // Load haystack once via the shared driver and keep the key.
    let hay_key: DesignKey =
        driver.ensure_loaded("examples/fixtures/basic/and/verilog/and_seq.v")?;

    // Build a QueryCtx for this netlist type from its metadata.
    let ctx = ctx_for::<AndGate<Search>>(&driver, &hay_key)?;

    let config = Config::builder().exact_length().none().build();

    let hits = AndGate::<Search>::query(&ctx, Instance::root("and".to_string()), &config);

    for hit in hits {
        println!("Found match: {:#?}", hit);
    }

    Ok(())
}
