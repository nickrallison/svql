use std::path::PathBuf;
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pattern_module: YosysModule =
        YosysModule::new("examples/patterns/basic/and/verilog/and_gate.v", "and_gate")?;
    // let design_module: YosysModule = YosysModule::new(
    //     "examples/fixtures/larger_designs/verilog/tech.rocksavage.chiselware.addrdecode.AddrDecode_64_64_64.v",
    //     "AddrDecode",
    // )?;

    let design_module: YosysModule =
        YosysModule::new("examples/patterns/basic/and/verilog/and_gate.v", "and_gate")?;

    let config = svql_common::Config::builder()
        .match_length(true)
        .dedupe(false)
        .haystack_flatten(true)
        .build();

    let yosys = PathBuf::from("/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys");

    let pattern = pattern_module.import_design_yosys(&config.needle_options, &yosys)?;
    let design = design_module.import_design_yosys(&config.haystack_options, &yosys)?;

    let matcher = svql_subgraph::FindSubgraphs::new(&pattern, &design, &config);

    let matches = matcher.find_subgraph_isomorphisms();

    for match_ in matches {
        match_.print_mapping();
    }

    // let pattern = pattern_module.
    // let design = design_module.get_design();

    Ok(())
}
