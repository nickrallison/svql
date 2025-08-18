

use svql_driver::{cache::Cache, subgraph::find_subgraphs, util::{load_driver_cached, load_driver_from}, Driver};
use svql_query::{instance::Instance, netlist::SearchableNetlist, queries::basic::and::And, Search};

lazy_static::lazy_static! {
    static ref AND_GATE: Driver = load_driver_from("examples/patterns/basic/and/and_gate.v").unwrap();
    static ref AND_TREE: Driver = load_driver_from("examples/patterns/basic/and/and_tree.v").unwrap();
    static ref AND_SEQ:  Driver = load_driver_from("examples/patterns/basic/and/and_seq.v").unwrap();
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let hits = And::<Search>::query(&*AND_GATE, &*AND_SEQ, Instance::root("and".to_string()));
    for hit in hits {
        println!("Found match: {:#?}", hit);
    }

    Ok(())
}
