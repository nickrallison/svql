use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    // 1. Load the design
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_not_deep.v",
        "cwe1234_not_deep",
    )?;
    let needle_module: YosysModule =
        YosysModule::new("examples/fixtures/basic/or/verilog/or_gate.v", "or_gate")?;

    // 2. Import the design
    let design = design_module.import_design(&Default::default())?;
    let needle = needle_module.import_design(&Default::default())?;

    // 3. Create the matcher
    let config = Default::default();
    let assignment_set = svql_subgraph::SubgraphMatcher::enumerate_all(
        &needle,
        &design,
        needle_module.module_name().to_owned(),
        design_module.module_name().to_owned(),
        &config,
    );

    for (match_idx, assignment) in assignment_set.items.iter().enumerate() {
        println!("--- Match {match_idx} ---");
        for (haystack_cell, needle_cell) in assignment.haystack_mapping() {
            println!(
                "Needle Cell [id={}]: {:?} -> Haystack Cell [id={}]: {:?}",
                needle_cell.debug_index(),
                needle_cell.get(),
                haystack_cell.debug_index(),
                haystack_cell.get()
            );
        }
    }
    // ...
    Ok(())
}
