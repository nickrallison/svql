use prjunnamed_netlist::{Cell, ControlNet};
use std::borrow::Cow;
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_combined.v",
        "cwe1234_combined",
    )?;

    let design = design_module.import_design(&svql_common::ModuleConfig::default())?;

    for (order_idx, cell_ref) in design.iter_cells_topo().enumerate() {
        let cell: Cow<'_, Cell> = cell_ref.get();

        println!(
            "[{:<3}] Index {:<3}: {:#?}",
            order_idx,
            cell_ref.debug_index(),
            cell,
            // fanin_indices
        );
    }

    Ok(())
}
