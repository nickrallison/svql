// use svql_driver::{cache::Cache, util::load_driver_cached};
// use svql_query::{Search, instance::Instance, queries::netlist::basic::and::and_gate::AndGate};
// use svql_subgraph::config::Config;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // env logger
//     env_logger::builder()
//         .filter_level(log::LevelFilter::Trace)
//         .init();

//     let mut cache = Cache::new();

//     let and_gate_driver =
//         load_driver_cached("examples/patterns/basic/and/verilog/and_gate.v", &mut cache)?;
//     let and_seq_driver =
//         load_driver_cached("examples/fixtures/basic/and/verilog/and_seq.v", &mut cache)?;

//     let config = Config::builder().exact_length().none().build();

//     let hits = AndGate::<Search>::query(
//         &and_gate_driver,
//         &and_seq_driver,
//         Instance::root("and".to_string()),
//         &config,
//     );

//     for hit in hits {
//         println!("Found match: {:#?}", hit);
//     }

//     Ok(())
// }

fn main() {
    todo!()
}
