use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    // 1. Load the design
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v",
        "cwe1234_multi_width",
    )?;
    let needle_module: YosysModule = YosysModule::new(
        "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
        "async_mux",
    )?;

    // 2. Import the design
    let design = design_module.import_design(&svql_common::ModuleConfig::default())?;
    let needle = needle_module.import_design(&svql_common::ModuleConfig::default())?;

    // 3. Create the matcher
    let config = svql_common::Config::builder()
        .match_length(svql_common::MatchLength::NeedleSubsetHaystack)
        .build();
    let assignment_set = svql_subgraph::SubgraphMatcher::enumerate_all(
        &needle,
        &design,
        needle_module.module_name().to_owned(),
        design_module.module_name().to_owned(),
        &config,
    );

    // Build indices for resolving CellIndex → CellWrapper
    let needle_index = svql_subgraph::GraphIndex::build(&needle);
    let haystack_index = svql_subgraph::GraphIndex::build(&design);

    for (match_idx, assignment) in assignment_set.items.iter().enumerate() {
        println!("--- Match {match_idx} ---");
        for (needle_idx, haystack_idx) in assignment.needle_mapping() {
            let needle_cell = needle_index.get_cell_by_index(*needle_idx);
            let haystack_cell = haystack_index.get_cell_by_index(*haystack_idx);
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
