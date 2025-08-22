use std::sync::Arc;

use prjunnamed_netlist::Design;
use rstest::rstest;
use svql_driver::{Context, Driver};

lazy_static::lazy_static! {
    static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");

    static ref AND_Q_DOUBLE_SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "and_q_double_sdffe".to_string()
        );
    static ref AND_GATE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "and_gate".to_string()
        );
    static ref AND_TREE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "and_tree".to_string()
        );
    static ref AND_SEQ_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "and_seq".to_string()
        );

    static ref CONTEXT: Context = {
        let mut ctx = Context::new();

        // Load all required designs
        let designs = vec![
            (&*AND_Q_DOUBLE_SDFFE_KEY, "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v"),
            (&*AND_GATE_KEY, "examples/patterns/basic/and/verilog/and_gate.v"),
            (&*AND_TREE_KEY, "examples/fixtures/basic/and/verilog/and_tree.v"),
            (&*AND_SEQ_KEY, "examples/fixtures/basic/and/verilog/and_seq.v"),
        ];

        for (key, path) in designs {
            let (_load_key, design) = DRIVER.get_or_load_design(path, key.module_name().to_string())
                .expect(&format!("Failed to load design: {}", path));
            ctx = ctx.with_design(key.clone(), design);
        }

        ctx
    };

    static ref AND_Q_DOUBLE_SDFFE: Arc<Design> =
        CONTEXT.get(&*AND_Q_DOUBLE_SDFFE_KEY)
        .expect("AND_Q_DOUBLE_SDFFE design not found")
        .clone();
    static ref AND_GATE: Arc<Design> =
        CONTEXT.get(&*AND_GATE_KEY)
        .expect("AND_GATE design not found")
        .clone();
    static ref AND_TREE: Arc<Design> =
        CONTEXT.get(&*AND_TREE_KEY)
        .expect("AND_TREE design not found")
        .clone();
    static ref AND_SEQ: Arc<Design> =
        CONTEXT.get(&*AND_SEQ_KEY)
        .expect("AND_SEQ design not found")
        .clone();

    static ref CONFIG: svql_common::Config = svql_common::Config::builder()
        .match_length(false)
        .none()
        .build();
}

#[rstest]
// AND_Q_DOUBLE_SDFFE Needle
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 2)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), AND_GATE.as_ref(), 0)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), AND_TREE.as_ref(), 0)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), AND_SEQ.as_ref(), 0)]
// AND_GATE Needle
#[case(AND_GATE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 2)]
#[case(AND_GATE.as_ref(), AND_GATE.as_ref(), 1)]
#[case(AND_GATE.as_ref(), AND_TREE.as_ref(), 7)]
#[case(AND_GATE.as_ref(), AND_SEQ.as_ref(), 7)]
// AND_TREE Needle
#[case(AND_TREE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 0)]
#[case(AND_TREE.as_ref(), AND_GATE.as_ref(), 0)]
#[case(AND_TREE.as_ref(), AND_TREE.as_ref(), 1)]
#[case(AND_TREE.as_ref(), AND_SEQ.as_ref(), 0)]
// AND_SEQ Needle
#[case(AND_SEQ.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 0)]
#[case(AND_SEQ.as_ref(), AND_GATE.as_ref(), 0)]
#[case(AND_SEQ.as_ref(), AND_TREE.as_ref(), 0)]
#[case(AND_SEQ.as_ref(), AND_SEQ.as_ref(), 1)]
fn test_subgraph_matches(
    #[case] needle: &Design,
    #[case] haystack: &Design,
    #[case] expected: usize,
) {
    let matches = svql_subgraph::find_subgraphs(needle, haystack, &CONFIG);

    assert_eq!(
        matches.len(),
        expected,
        "Expected {} matches, got {}",
        expected,
        matches.len()
    );
}
