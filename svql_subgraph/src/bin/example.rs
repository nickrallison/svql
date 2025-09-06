use std::path::PathBuf;
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pattern_module: YosysModule = YosysModule::new(
        "examples/fixtures/basic/and/verilog/small_and_seq.v",
        "small_and_seq",
    )?;
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/basic/and/verilog/small_and_tree.v",
        "small_and_tree",
    )?;

    let config = svql_common::Config::builder()
        .match_length(true)
        .dedupe(false)
        .haystack_flatten(true)
        .build();

    let yosys = PathBuf::from("/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys");

    let pattern = pattern_module.import_design_yosys(&config.needle_options, &yosys)?;
    let design = design_module.import_design_yosys(&config.haystack_options, &yosys)?;

    let matches = svql_subgraph::find_subgraph_isomorphisms(&pattern, &design, &config);

    for match_ in matches {
        match_.print_mapping();
    }

    // let pattern = pattern_module.
    // let design = design_module.get_design();

    Ok(())
}
