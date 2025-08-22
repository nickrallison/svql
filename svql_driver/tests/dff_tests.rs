use std::path::PathBuf;
use std::sync::Arc;

use prjunnamed_netlist::Design;
use rstest::rstest;
use svql_driver::{Context, Driver};

lazy_static::lazy_static! {
    static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");

    static ref COMB_D_DOUBLE_SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
            "comb_d_double_sdffe".to_string()
        );
    static ref AND_Q_DOUBLE_SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "and_q_double_sdffe".to_string()
        );
    static ref PAR_DOUBLE_SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
            "par_double_sdffe".to_string()
        );
    static ref SEQ_DOUBLE_SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
            "seq_double_sdffe".to_string()
        );
    static ref SDFFE_KEY: svql_driver::DriverKey =
        svql_driver::DriverKey::new(
            "examples/patterns/basic/ff/verilog/sdffe.v",
            "sdffe".to_string()
        );

    static ref CONTEXT: Context = {
        let mut ctx = Context::new();

        // Load all required designs
        let designs = vec![
            (&*COMB_D_DOUBLE_SDFFE_KEY, "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v"),
            (&*AND_Q_DOUBLE_SDFFE_KEY, "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v"),
            (&*PAR_DOUBLE_SDFFE_KEY, "examples/fixtures/basic/ff/verilog/par_double_sdffe.v"),
            (&*SEQ_DOUBLE_SDFFE_KEY, "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v"),
            (&*SDFFE_KEY, "examples/patterns/basic/ff/verilog/sdffe.v"),
        ];

        for (key, path) in designs {
            let (_load_key, design) = DRIVER.get_or_load_design(path, key.module_name().to_string())
                .expect(&format!("Failed to load design: {}", path));
            ctx = ctx.with_design(key.clone(), design);
        }

        ctx
    };

    static ref COMB_D_DOUBLE_SDFFE: Arc<Design> =
        CONTEXT.get(&*COMB_D_DOUBLE_SDFFE_KEY)
        .expect("COMB_D_DOUBLE_SDFFE design not found")
        .clone();
    static ref AND_Q_DOUBLE_SDFFE: Arc<Design> =
        CONTEXT.get(&*AND_Q_DOUBLE_SDFFE_KEY)
        .expect("AND_Q_DOUBLE_SDFFE design not found")
        .clone();
    static ref PAR_DOUBLE_SDFFE: Arc<Design> =
        CONTEXT.get(&*PAR_DOUBLE_SDFFE_KEY)
        .expect("PAR_DOUBLE_SDFFE design not found")
        .clone();
    static ref SEQ_DOUBLE_SDFFE: Arc<Design> =
        CONTEXT.get(&*SEQ_DOUBLE_SDFFE_KEY)
        .expect("SEQ_DOUBLE_SDFFE design not found")
        .clone();
    static ref SDFFE: Arc<Design> =
        CONTEXT.get(&*SDFFE_KEY)
        .expect("SDFFE design not found")
        .clone();

    static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
        .match_length(false)
        .none()
        .build();
}

#[rstest]
// Comb D Needle
#[case(COMB_D_DOUBLE_SDFFE.as_ref(), COMB_D_DOUBLE_SDFFE.as_ref(), 2)]
#[case(COMB_D_DOUBLE_SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 0)]
#[case(COMB_D_DOUBLE_SDFFE.as_ref(), PAR_DOUBLE_SDFFE.as_ref(), 0)]
#[case(COMB_D_DOUBLE_SDFFE.as_ref(), SEQ_DOUBLE_SDFFE.as_ref(), 0)]
#[case(COMB_D_DOUBLE_SDFFE.as_ref(), SDFFE.as_ref(), 0)]
// And Q Needle
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), COMB_D_DOUBLE_SDFFE.as_ref(), 0)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 2)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), PAR_DOUBLE_SDFFE.as_ref(), 0)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), SEQ_DOUBLE_SDFFE.as_ref(), 0)]
#[case(AND_Q_DOUBLE_SDFFE.as_ref(), SDFFE.as_ref(), 0)]
// Par Double Needle
#[case(PAR_DOUBLE_SDFFE.as_ref(), COMB_D_DOUBLE_SDFFE.as_ref(), 2)]
#[case(PAR_DOUBLE_SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 2)]
#[case(PAR_DOUBLE_SDFFE.as_ref(), PAR_DOUBLE_SDFFE.as_ref(), 2)]
#[case(PAR_DOUBLE_SDFFE.as_ref(), SEQ_DOUBLE_SDFFE.as_ref(), 2)]
#[case(PAR_DOUBLE_SDFFE.as_ref(), SDFFE.as_ref(), 0)]
// Seq Double Needle
#[case(SEQ_DOUBLE_SDFFE.as_ref(), COMB_D_DOUBLE_SDFFE.as_ref(), 0)]
#[case(SEQ_DOUBLE_SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 0)]
#[case(SEQ_DOUBLE_SDFFE.as_ref(), PAR_DOUBLE_SDFFE.as_ref(), 0)]
#[case(SEQ_DOUBLE_SDFFE.as_ref(), SEQ_DOUBLE_SDFFE.as_ref(), 1)]
#[case(SEQ_DOUBLE_SDFFE.as_ref(), SDFFE.as_ref(), 0)]
// SDFFE Needle
#[case(SDFFE.as_ref(), COMB_D_DOUBLE_SDFFE.as_ref(), 2)]
#[case(SDFFE.as_ref(), AND_Q_DOUBLE_SDFFE.as_ref(), 2)]
#[case(SDFFE.as_ref(), PAR_DOUBLE_SDFFE.as_ref(), 2)]
#[case(SDFFE.as_ref(), SEQ_DOUBLE_SDFFE.as_ref(), 2)]
#[case(SDFFE.as_ref(), SDFFE.as_ref(), 1)]
fn test_sdffe_subgraph_matches(
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
