use prjunnamed_netlist::Cell;
use std::{borrow::Cow, path::PathBuf};
use svql_common::YosysModule;
use svql_design_set::{DesignSet, design_container::DesignContainer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let needle_module: YosysModule =
        YosysModule::new("examples/patterns/basic/and/verilog/and_gate.v", "and_gate")?;

    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/larger_designs/json/openpiton_chip_full.json",
        "chip",
    )?;

    // let design_module: YosysModule = YosysModule::new(
    //     "examples/fixtures/larger_designs/json/otbn_core.json",
    //     "otbn_core",
    // )?;

    // let design_module: YosysModule = YosysModule::new(
    //     "examples/fixtures/larger_designs/verilog/tech.rocksavage.chiselware.addrdecode.AddrDecode_64_64_64.v",
    //     "AddrDecode",
    // )?;

    // let design_module: YosysModule =
    //     YosysModule::new("examples/needles/basic/and/verilog/and_gate.v", "and_gate")?;

    let module_config = svql_common::ModuleConfig {
        flatten: false,
        verific: true,
        ..Default::default()
    };

    let config = svql_common::Config::builder()
        .match_length(svql_common::MatchLength::First)
        .dedupe(svql_common::Dedupe::Inner)
        .haystack_flatten(false)
        .haystack_options(module_config.clone())
        .needle_options(module_config)
        .build();

    let yosys = PathBuf::from("/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys");

    let needle_set = needle_module.import_design_yosys(&config.needle_options, &yosys)?;
    // let design_set = design_module.import_design_yosys(&config.haystack_options, &yosys)?;
    let design_set = design_module.import_design_raw()?;

    let needle = needle_set
        .1
        .get("and_gate")
        .ok_or("Needle module not found")?;

    let needle_container = DesignContainer::build(needle.clone());

    let haystack_key = design_set.0.clone();
    let haystack_set = DesignSet::from_designs(design_set.0, design_set.1);

    let subgraph_matcher = svql_subgraph::SubgraphMatcher::new(
        &needle_container,
        haystack_key,
        &haystack_set,
        &config,
    );

    let embeddings = subgraph_matcher.get_matches();

    println!("Found {} embeddings", embeddings.items.len());

    Ok(())
}
