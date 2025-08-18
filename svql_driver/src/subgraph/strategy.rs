use super::index::Index;
use super::index::NodeId;
use super::ports::Source;
use super::state::State;

// Choose the next unmapped pattern node to extend, preferring nodes whose inputs
// are constants, IO, or mapped gate sources (i.e., “resolvable”).
pub(super) fn choose_next<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>) -> Option<NodeId> {
    // Prefer nodes with all inputs “resolvable”
    for p in 0..(p_index.gate_count() as usize) {
        let p = p as NodeId;
        if st.is_mapped(p) { continue; }

        let pins = &p_index.pins(p).inputs;
        let mut all_resolvable = true;
        for (_, src) in pins {
            match src {
                Source::Const(_) => {}
                Source::Io(_, _) => {}
                Source::Gate(gc, _) => {
                    if let Some(g) = p_index.try_cell_to_node(*gc) {
                        if !st.is_mapped(g) {
                            all_resolvable = false;
                            break;
                        }
                    }
                }
            }
        }
        if all_resolvable { return Some(p); }
    }

    // Fallback: any unmapped
    for p in 0..(p_index.gate_count() as usize) {
        let p = p as NodeId;
        if !st.is_mapped(p) {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{Driver};
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref SDFFE: (Driver, String) = load_driver_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn choose_next_returns_some() {
        let d = SDFFE.0.design_as_ref();
        let idx = Index::build(d);
        let st = State::new(idx.gate_count());
        assert!(choose_next(&idx, &st).is_some());
    }
}