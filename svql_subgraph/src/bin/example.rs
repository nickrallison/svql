use prjunnamed_netlist::{Cell, ControlNet};
use std::{borrow::Cow, path::PathBuf};
use svql_common::YosysModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the design
    let design_module: YosysModule = YosysModule::new(
        "examples/fixtures/basic/ff/verilog/dff_loop_toggle.v",
        "dff_loop_toggle",
    )?;

    let module_config = svql_common::ModuleConfig {
        flatten: false,
        verific: false,
        ..Default::default()
    };

    // 2. Import the design
    let yosys = PathBuf::from("yosys");
    let design = design_module.import_design_yosys(&module_config, &yosys)?;

    println!("--- Verifying Topological Sort Order ---");
    println!("Expectation: Inputs/DFFs (Boundaries) -> Combinational Logic -> Outputs");
    println!("Format: [Order] Index: Type <- Fanin[Indices]");
    println!("------------------------------------------------------------------------");

    // Helper to resolve a Value (vector of nets) to source cell indices
    let resolve_value = |val: &prjunnamed_netlist::Value| -> Vec<String> {
        val.iter()
            .filter_map(|net| {
                design
                    .find_cell(net)
                    .ok()
                    .map(|(source_ref, _)| format!("{}", source_ref.debug_index()))
            })
            .collect()
    };

    // Helper to resolve a single Net to source cell index
    let resolve_net = |net: prjunnamed_netlist::Net| -> Vec<String> {
        design
            .find_cell(net)
            .ok()
            .map(|(source_ref, _)| vec![format!("{}", source_ref.debug_index())])
            .unwrap_or_default()
    };

    // Helper to resolve ControlNet
    let resolve_control = |cnet: &ControlNet| -> Vec<String> {
        match cnet {
            ControlNet::Pos(n) | ControlNet::Neg(n) => resolve_net(*n),
        }
    };

    // 3. Iterate using the topological iterator
    for (order_idx, cell_ref) in design.iter_cells_topo().enumerate() {
        let cell: Cow<'_, Cell> = cell_ref.get();

        // Extract Fanin Indices based on cell type
        let fanin_indices = match &*cell {
            Cell::And(a, b)
            | Cell::Or(a, b)
            | Cell::Xor(a, b)
            | Cell::Eq(a, b)
            | Cell::ULt(a, b)
            | Cell::SLt(a, b) => {
                let mut deps = resolve_value(a);
                deps.extend(resolve_value(b));
                deps
            }
            Cell::Not(a) | Cell::Buf(a) => resolve_value(a),
            Cell::Mux(sel, a, b) => {
                let mut deps = resolve_net(*sel);
                deps.extend(resolve_value(a));
                deps.extend(resolve_value(b));
                deps
            }
            Cell::Dff(dff) => {
                // For DFFs, we care about Data and Enable for dependency checking
                let mut deps = resolve_value(&dff.data);
                let en: &ControlNet = &dff.enable;
                deps.extend(resolve_control(en));
                deps
            }
            Cell::Output(_, val) => resolve_value(val),
            _ => vec![],
        };

        let cell_type = match &*cell {
            Cell::Input(name, _) => format!("Input({})", name),
            Cell::Output(name, _) => format!("Output({})", name),
            Cell::Dff(_) => "DFF".to_string(),
            Cell::And(_, _) => "AND".to_string(),
            Cell::Or(_, _) => "OR".to_string(),
            Cell::Not(_) => "NOT".to_string(),
            Cell::Mux(_, _, _) => "MUX".to_string(),
            _ => format!("{:?}", cell)
                .split('(')
                .next()
                .unwrap_or("Unknown")
                .to_string(),
        };

        println!(
            "[{:<3}] Index {:<3}: {:<15} <- Fanin{:?}",
            order_idx,
            cell_ref.debug_index(),
            cell_type,
            fanin_indices
        );
    }

    Ok(())
}
