use prjunnamed_netlist::{Design, Net, Cell};
use crate::subgraph::{SubgraphMatch, cell_kind::CellWrapper};

/// Check if an output port of one match connects directly (same net) to an input port of another match
pub fn check_port_connection<'p, 'd>(
    source_match: &SubgraphMatch<'p, 'd>,
    source_output_name: &str,
    target_match: &SubgraphMatch<'p, 'd>,
    target_input_name: &str,
    _design: &'d Design,
) -> bool {
    // Get the design nets that drive the source output (on the haystack)
    let source_nets = get_output_driver_nets(source_match, source_output_name);
    // Get the design nets that the target input is bound to (on the haystack)
    let target_nets = get_input_connection_nets(target_match, target_input_name);

    match (source_nets, target_nets) {
        (Some(src_nets), Some(tgt_nets)) => {
            src_nets.iter().any(|src| tgt_nets.iter().any(|tgt| src == tgt))
        }
        _ => false,
    }
}

/// Get the design nets (haystack) that drive a named pattern output (needle)
fn get_output_driver_nets<'p, 'd>(
    m: &SubgraphMatch<'p, 'd>,
    output_name: &str,
) -> Option<Vec<Net>> {
    // 1) Find the pattern Output cell by name
    let pat_output = m.pat_output_cells.iter()
        .find(|out| out.name() == Some(output_name))?;

    // 2) Extract the pattern Output's input value (the thing driving the output)
    let cell_ref = pat_output.cref.cref().get();
    let cell = cell_ref.as_ref();
    let Cell::Output(_, value) = cell else {
        return None;
    };

    let mut design_nets = Vec::new();

    // 3) For each bit of that value, resolve the producing pattern cell+bit,
    //    then map that cell to the matched design cell, and take the same output bit there.
    for net in value.iter() {
        // pattern side: who produces this net?
        if let Ok((p_src_cell_ref, p_bit)) = pat_output.cref.design().find_cell(net) {
            let p_src = CellWrapper::from(p_src_cell_ref);
            if let Some(&d_src) = m.cell_mapping.get(&p_src) {
                let d_out = d_src.output();
                if p_bit < d_out.len() {
                    design_nets.push(d_out[p_bit]);
                }
            }
        } else {
            // net is constant or undef on the pattern side — no mapped design net
        }
    }

    if design_nets.is_empty() { None } else { Some(design_nets) }
}

/// Get the design nets (haystack) that a named pattern input (needle) is bound to
fn get_input_connection_nets<'p, 'd>(
    m: &SubgraphMatch<'p, 'd>,
    input_name: &str,
) -> Option<Vec<Net>> {
    // 1) Find the pattern Input cell by name
    let pat_input = m.pat_input_cells.iter()
        .find(|inp| inp.name() == Some(input_name))?;

    // 2) Determine the width of that input on the pattern side
    let width = match pat_input.cref.get().as_ref() {
        Cell::Input(_, w) => *w,
        _ => 0,
    };
    if width == 0 { return None; }

    // 3) For each bit, consult boundary_src_map: it binds (pattern IO cell, bit)
    //    to (design cell, bit). Then take the corresponding design net.
    let mut design_nets = Vec::new();
    for b in 0..width {
        if let Some(&(d_cell, d_bit)) = m.boundary_src_map.get(&(pat_input.cref, b)) {
            let d_out = d_cell.output();
            if d_bit < d_out.len() {
                design_nets.push(d_out[d_bit]);
            }
        }
    }

    if design_nets.is_empty() { None } else { Some(design_nets) }
}

/// Simple helper for common case
pub fn is_connected<'p, 'd>(
    match1: &SubgraphMatch<'p, 'd>,
    output_name: &str,
    match2: &SubgraphMatch<'p, 'd>,
    input_name: &str,
    design: &'d Design,
) -> bool {
    check_port_connection(match1, output_name, match2, input_name, design)
}