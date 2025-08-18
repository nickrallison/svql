use svql_driver::{cache::Cache, util::load_driver_cached};
use svql_query::{Search, instance::Instance, queries::netlist::and::And};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = Cache::new();

    let and_gate_driver = load_driver_cached("examples/patterns/basic/and/and_gate.v", &mut cache)?;
    let and_seq_driver = load_driver_cached("examples/patterns/basic/and/and_seq.v", &mut cache)?;

    let hits = And::<Search>::query(
        &and_gate_driver,
        &and_seq_driver,
        Instance::root("and".to_string()),
    );

    for hit in hits {
        println!("Found match: {:#?}", hit);
    }

    Ok(())
}
