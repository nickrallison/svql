use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the design
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json",
        "tile",
    )?;
    let needle_module: YosysModule =
        YosysModule::new("examples/patterns/basic/and/json/and_gate.json", "and_gate")?;

    // 2. Import the design
    let design = design_module.import_design_raw()?;
    let needle = needle_module.import_design_raw()?;

    // 3. Create the matcher
    let config = svql_common::Config {
        ..Default::default()
    };
    let assignment_set = svql_subgraph::SubgraphMatcher::enumerate_all(
        &needle,
        &design,
        needle_module.module_name().to_string(),
        design_module.module_name().to_string(),
        &config,
    );

    for (match_idx, assignment) in assignment_set.items.iter().enumerate() {
        if match_idx % 1000 != 0 {
            continue;
        }
        println!("--- Match {} ---", match_idx);
        for (haystack_cell, needle_cell) in assignment.haystack_mapping().iter() {
            let needle_cell = needle_cell.get();
            let haystack_cell = haystack_cell.get();
            println!(
                "Needle Cell: {:?} -> Haystack Cell: {:?}",
                needle_cell, haystack_cell
            );
        }
    }
    // ...
    Ok(())
}
