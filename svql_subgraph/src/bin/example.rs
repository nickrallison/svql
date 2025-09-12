use std::path::PathBuf;
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pattern_module: YosysModule = YosysModule::new(
        "examples/patterns/basic/and/verilog/and_2_seq.v",
        "and_2_seq",
    )?;

    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/larger_designs/json/otbn_core.json",
        "otbn_core",
    )?;

    // let design_module: YosysModule = YosysModule::new(
    //     "examples/fixtures/larger_designs/verilog/tech.rocksavage.chiselware.addrdecode.AddrDecode_64_64_64.v",
    //     "AddrDecode",
    // )?;

    // let design_module: YosysModule =
    //     YosysModule::new("examples/patterns/basic/and/verilog/and_gate.v", "and_gate")?;

    let config = svql_common::Config::builder()
        .match_length(svql_common::MatchLength::First)
        .dedupe(svql_common::Dedupe::Inner)
        .haystack_flatten(true)
        .build();

    let yosys = PathBuf::from("/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys");

    let pattern = pattern_module.import_design_yosys(&config.needle_options, &yosys)?;
    let design = design_module.import_design_yosys(&config.haystack_options, &yosys)?;

    let matches = svql_subgraph::FindSubgraphs::find_subgraphs(&pattern, &design, &config);

    // for match_ in matches {
    //     match_.print_mapping();
    // }

    println!("Found {} matches", matches.len());

    // let pattern = pattern_module.
    // let design = design_module.get_design();

    Ok(())
}
