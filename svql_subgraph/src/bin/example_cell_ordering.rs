//! Binary utility for inspecting the topological ordering of cells within a netlist.

use prjunnamed_netlist::Cell;
use std::borrow::Cow;
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let design_module: YosysModule = YosysModule::new(
        "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
        "async_en",
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
