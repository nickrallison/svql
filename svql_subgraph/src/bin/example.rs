use prjunnamed_netlist::Cell;
use std::{borrow::Cow, path::PathBuf};
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let needle_module: YosysModule =
        YosysModule::new("examples/patterns/basic/and/verilog/and_gate.v", "and_gate")?;

    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/larger_designs/json/openpiton_tile_full.json",
        "tile",
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
        verific: false,
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

    let needle = needle_module.import_design_yosys(&config.needle_options, &yosys)?;
    let design = design_module.import_design_raw()?;

    let embeddings = svql_subgraph::SubgraphMatcher::enumerate_all(&needle, &design, &config);

    println!("Found {} embeddings", embeddings.items.len());

    Ok(())
}
