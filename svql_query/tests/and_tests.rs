use svql_driver::prelude::Driver;
use svql_driver::util::load_driver_from;

use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::queries::basic::and::And;

lazy_static::lazy_static! {
    static ref AND_GATE: Driver = load_driver_from("examples/patterns/basic/and/and_gate.v").unwrap();
    static ref AND_TREE: Driver = load_driver_from("examples/patterns/basic/and/and_tree.v").unwrap();
    static ref AND_SEQ:  Driver = load_driver_from("examples/patterns/basic/and/and_seq.v").unwrap();
}

fn root_instance() -> Instance {
    Instance::root("and".to_string())
}

#[test]
fn and_counts_and_gate_vs_and_gate() {
    let hits = And::<Search>::query(&*AND_GATE, &*AND_GATE, root_instance());
    assert_eq!(hits.len(), 1, "and_gate vs and_gate should yield 1 match");
}

#[test]
fn and_counts_and_gate_vs_and_tree() {
    let hits = And::<Search>::query(&*AND_GATE, &*AND_TREE, root_instance());
    assert_eq!(hits.len(), 7, "and_gate vs and_tree should yield 7 matches");
}

#[test]
fn and_counts_and_gate_vs_and_seq() {
    let hits = And::<Search>::query(&*AND_GATE, &*AND_SEQ, root_instance());
    assert_eq!(hits.len(), 7, "and_gate vs and_seq should yield 7 matches");
}

#[test]
fn and_counts_and_tree_vs_and_tree() {
    // Using the And netlist against a multi-gate pattern is allowed because the pattern has ports a,b,y.
    let hits = And::<Search>::query(&*AND_TREE, &*AND_TREE, root_instance());
    assert_eq!(hits.len(), 1, "and_tree vs and_tree should yield 1 match");
}

#[test]
fn and_counts_and_seq_vs_and_seq() {
    let hits = And::<Search>::query(&*AND_SEQ, &*AND_SEQ, root_instance());
    assert_eq!(hits.len(), 1, "and_seq vs and_seq should yield 1 match");
}

#[test]
fn and_counts_and_tree_vs_and_gate_is_zero() {
    let hits = And::<Search>::query(&*AND_TREE, &*AND_GATE, root_instance());
    assert_eq!(hits.len(), 0, "and_tree vs and_gate should yield 0 matches");
}

#[test]
fn and_counts_and_seq_vs_and_gate_is_zero() {
    let hits = And::<Search>::query(&*AND_SEQ, &*AND_GATE, root_instance());
    assert_eq!(hits.len(), 0, "and_seq vs and_gate should yield 0 matches");
}

#[test]
fn and_bindings_present_and_gate_vs_and_tree() {
    // For each match, confirm a, b, y have bound design cells (like the example_driver pattern)
    let hits = And::<Search>::query(&*AND_GATE, &*AND_TREE, root_instance());
    assert!(!hits.is_empty());

    for h in &hits {
        let a = h.a.val.as_ref().expect("missing a");
        let b = h.b.val.as_ref().expect("missing b");
        let y = h.y.val.as_ref().expect("missing y");

        assert!(
            a.pat_cell_ref.is_some(),
            "pattern cell for a should be present"
        );
        assert!(
            b.pat_cell_ref.is_some(),
            "pattern cell for b should be present"
        );
        assert!(
            y.pat_cell_ref.is_some(),
            "pattern cell for y should be present"
        );

        assert!(
            a.design_cell_ref.is_some(),
            "design source for a should be bound"
        );
        assert!(
            b.design_cell_ref.is_some(),
            "design source for b should be bound"
        );
        assert!(
            y.design_cell_ref.is_some(),
            "design driver for y should be bound"
        );
    }
}

fn any_connection_exists(hits: &[svql_query::queries::basic::and::And<svql_query::Match>]) -> bool {
    for left in hits {
        if let Some(lhs_y_cell) = left.y.val.as_ref().and_then(|m| m.design_cell_ref) {
            let lhs_net = lhs_y_cell.output()[0];
            for right in hits {
                if let Some(rhs_a_cell) = right.a.val.as_ref().and_then(|m| m.design_cell_ref) {
                    if lhs_net == rhs_a_cell.output()[0] {
                        return true;
                    }
                }
                if let Some(rhs_b_cell) = right.b.val.as_ref().and_then(|m| m.design_cell_ref) {
                    if lhs_net == rhs_b_cell.output()[0] {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[test]
fn and_connectivity_exists_in_and_tree() {
    let hits = And::<Search>::query(&*AND_GATE, &*AND_TREE, root_instance());
    assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

    let connected = any_connection_exists(&hits);
    assert!(
        connected,
        "expected at least one connection y->(a|b) among matches in and_tree"
    );
}

#[test]
fn and_connectivity_exists_in_and_seq() {
    let hits = And::<Search>::query(&*AND_GATE, &*AND_SEQ, root_instance());
    assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

    let connected = any_connection_exists(&hits);
    assert!(
        connected,
        "expected at least one connection y->(a|b) among matches in and_seq"
    );
}
