use super::index::Index;
use super::index::NodeId;
use super::ports::Source;
use super::state::State;

pub(super) fn choose_next<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>) -> Option<NodeId> {
    let first_resolvable = (0..p_index.gate_count() as u32)
        .map(|i| i as NodeId)
        .find(|&p| !st.is_mapped(p) && inputs_resolved_for(p_index, st, p));

    first_resolvable.or_else(|| {
        (0..p_index.gate_count() as u32)
            .map(|i| i as NodeId)
            .find(|&p| !st.is_mapped(p))
    })
}

fn inputs_resolved_for<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>, p: NodeId) -> bool {
    p_index.pins(p).inputs.iter().all(|src| match src {
        Source::Const(_) => true,
        Source::Io(_, _) => true,
        Source::Gate(gc, _) => p_index
            .try_cell_to_node(*gc)
            .map_or(false, |g| st.is_mapped(g)),
    })
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn choose_next_returns_some() {
        let d = &SDFFE;
        let idx = Index::build(d);
        let st = State::new(idx.gate_count());
        assert!(choose_next(&idx, &st).is_some());
    }
}
