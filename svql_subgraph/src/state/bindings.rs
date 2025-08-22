use crate::index::{Index, NodeId};
use crate::model::Source;

use super::{State, align::aligned_sources};

use prjunnamed_netlist::Trit;

/// Self-documenting wrapper for an aligned pattern/design input pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AlignedPair<'p, 'd> {
    pub pattern: Source<'p>,
    pub design: Source<'d>,
}

/// Self-documenting wrapper for one binding addition (instead of tuple typing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindingAddition<'p, 'd> {
    pub pattern: PatSrcKey<'p>,
    pub design: DesSrcKey<'d>,
}

/// Alias to keep signatures short and clear
pub(crate) type BindingAdditions<'p, 'd> = Vec<BindingAddition<'p, 'd>>;

/// A canonical representation of a pattern driver bit used by some sink pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PatSrcKey<'p> {
    #[allow(dead_code)]
    Gate { node: NodeId, bit: usize },
    External {
        cell: crate::model::CellWrapper<'p>,
        bit: usize,
    },
    #[allow(dead_code)]
    Const(Trit),
}

/// A canonical representation of a design driver bit used by some sink pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DesSrcKey<'d> {
    Gate {
        node: NodeId,
        bit: usize,
    },
    External {
        cell: crate::model::CellWrapper<'d>,
        bit: usize,
    },
    #[allow(dead_code)]
    Const(Trit),
}

/// Validate aligned sources pairwise and collect any driver bindings implied.
/// Returns additions to apply if compatible.
#[contracts::debug_ensures(ret.is_none() || ret.as_ref().unwrap().iter().all(|a| st.binding_get(a.pattern).is_none()))]
pub(crate) fn check_and_collect_bindings<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &State<'p, 'd>,
    match_length: bool,
) -> Option<BindingAdditions<'p, 'd>> {
    let pairs = aligned_sources(p_id, d_id, p_index, d_index, match_length)?;
    let mut additions: BindingAdditions<'p, 'd> = Vec::new();

    for AlignedPair {
        pattern: p_src,
        design: d_src,
    } in pairs
    {
        match (p_src, d_src) {
            (Source::Const(pc), Source::Const(dc)) => {
                if pc != dc {
                    return None;
                }
            }
            (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let p_node = p_index.try_cell_to_node(p_cell)?;
                let d_node = d_index.try_cell_to_node(d_cell)?;
                if !super::constraints::mapped_gate_pair_ok(st, p_node, p_bit, d_node, d_bit) {
                    return None;
                }
            }
            (Source::Io(p_cell, p_bit), d_src @ (Source::Gate(_, _) | Source::Io(_, _))) => {
                let p_key = PatSrcKey::External {
                    cell: p_cell,
                    bit: p_bit,
                };
                let d_key = des_key_from_gate_or_io(d_index, d_src)?;
                if !unify_external_binding(st, &mut additions, p_key, d_key) {
                    return None;
                }
            }
            _ => return None,
        }
    }

    Some(additions)
}

/// Convert a design-side Source (Gate or Io) into a DesSrcKey.
/// Returns None if the source is not supported in this context (e.g., Const).
#[contracts::debug_ensures(ret.is_some() || matches!(d_src, Source::Const(_)))]
pub(crate) fn des_key_from_gate_or_io<'d>(
    d_index: &Index<'d>,
    d_src: Source<'d>,
) -> Option<DesSrcKey<'d>> {
    match d_src {
        Source::Gate(d_cell, d_bit) => {
            let d_node = d_index.try_cell_to_node(d_cell)?;
            Some(DesSrcKey::Gate {
                node: d_node,
                bit: d_bit,
            })
        }
        Source::Io(d_cell, d_bit) => Some(DesSrcKey::External {
            cell: d_cell,
            bit: d_bit,
        }),
        _ => None,
    }
}

/// Insert-or-validate a binding for a pattern External source.
/// - If a binding exists, it must match d_key.
/// - If not, record it in additions (to be inserted by the caller later).
pub(crate) fn unify_external_binding<'p, 'd>(
    st: &State<'p, 'd>,
    additions: &mut BindingAdditions<'p, 'd>,
    p_key: PatSrcKey<'p>,
    d_key: DesSrcKey<'d>,
) -> bool {
    match st.binding_get(p_key) {
        Some(existing) => existing == d_key,
        None => {
            additions.push(BindingAddition {
                pattern: p_key,
                design: d_key,
            });
            true
        }
    }
}
