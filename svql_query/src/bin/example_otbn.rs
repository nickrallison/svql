use svql_driver::{cache::Cache, util::load_driver_cached};
use svql_query::{
    Search,
    instance::Instance,
    queries::netlist::basic::{and::and_gate::AndGate, dff::Sdffe},
};
use svql_subgraph::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = Cache::new();

    let otbn_driver = load_driver_cached(
        "examples/fixtures/larger_designs/json/otbn_core.json",
        &mut cache,
    )?;
    let and_driver =
        load_driver_cached("examples/patterns/basic/and/verilog/and_gate.v", &mut cache)?;

    let config = Config::builder().exact_length().none().build();

    let hits = AndGate::<Search>::query(
        &and_driver,
        &otbn_driver,
        Instance::root("and_gate".to_string()),
        &config,
    );

    for hit in hits {
        println!("Found match: {:#?}", hit);
    }

    Ok(())
}
