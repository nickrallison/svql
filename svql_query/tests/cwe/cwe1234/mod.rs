mod unlock_logic;

use std::sync::OnceLock;

use common::{Config, Dedupe, MatchLength, YosysModule};
use driver::Driver;
use svql_query::security::cwe1234::unlock_logic::UnlockLogic;
use svql_query::traits::{Query, Searchable};
use svql_query::{Search, instance::Instance};

#[derive(Debug, Clone)]
struct CweTestCase {
    name: &'static str,
    fixture_path: &'static str,
    module_name: &'static str,
    description: &'static str,
    expected_matches: usize,
    min_or_depth: Option<usize>,
}

static CWE1234_CASES: &[CweTestCase] = &[
    CweTestCase {
        name: "simple",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_simple.v",
        module_name: "cwe1234_simple",
        description: "Minimal CWE1234 pattern: write & (~lock | debug)",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "swapped",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_swapped.v",
        module_name: "cwe1234_swapped",
        description: "Swapped inputs: (~lock | scan | debug) & write",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "sync_reset",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v",
        module_name: "cwe1234_sync_reset",
        description: "Synchronous reset style with bypass",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "wide_reg",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v",
        module_name: "cwe1234_wide_reg",
        description: "32-bit wide register with bypass",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "not_deep",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_not_deep.v",
        module_name: "cwe1234_not_deep",
        description: "NOT gate at varying nesting depths (stress test)",
        expected_matches: 3,
        min_or_depth: Some(3),
    },
    CweTestCase {
        name: "not_positions",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_not_positions.v",
        module_name: "cwe1234_not_positions",
        description: "NOT gate at different horizontal positions in OR tree",
        expected_matches: 5,
        min_or_depth: None,
    },
    CweTestCase {
        name: "not_right",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_not_right.v",
        module_name: "cwe1234_not_right",
        description: "NOT gate consistently on right side of OR gates",
        expected_matches: 3,
        min_or_depth: None,
    },
    CweTestCase {
        name: "not_alternating",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v",
        module_name: "cwe1234_not_alternating",
        description: "NOT gate alternates left/right at each depth",
        expected_matches: 4,
        min_or_depth: None,
    },
    CweTestCase {
        name: "multi_reg",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v",
        module_name: "cwe1234_multi_reg",
        description: "Multiple vulnerable registers with different bypass combinations",
        expected_matches: 3,
        min_or_depth: None,
    },
    CweTestCase {
        name: "multi_width",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v",
        module_name: "cwe1234_multi_width",
        description: "Multiple registers with different bit widths (1/8/16/32-bit)",
        expected_matches: 4,
        min_or_depth: None,
    },
    CweTestCase {
        name: "no_reset",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v",
        module_name: "cwe1234_no_reset",
        description: "DFFs without reset (minimal structure)",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "mixed_resets",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v",
        module_name: "cwe1234_mixed_resets",
        description: "Mix of async and sync reset DFFs",
        expected_matches: 2,
        min_or_depth: None,
    },
    CweTestCase {
        name: "combined",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_combined.v",
        module_name: "cwe1234_combined",
        description: "Wrong unlock: write & ((~lock & mode_a) | debug)",
        expected_matches: 0,
        min_or_depth: None,
    },
    CweTestCase {
        name: "deep",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_deep.v",
        module_name: "cwe1234_deep",
        description: "Deep OR tree with 4 bypass conditions",
        expected_matches: 1,
        min_or_depth: Some(3),
    },
    CweTestCase {
        name: "enabled",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_enabled.v",
        module_name: "cwe1234_enabled",
        description: "DFFs with explicit enable signals",
        expected_matches: 1,
        min_or_depth: None,
    },
    CweTestCase {
        name: "fixed",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_fixed.v",
        module_name: "cwe1234_fixed",
        description: "FIXED version - no bypass (secure)",
        expected_matches: 0,
        min_or_depth: None,
    },
];

fn init_test_logger() {
    static INIT: OnceLock<()> = OnceLock::new();
    let _ = INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

#[test]
fn test_cwe1234_variants() {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace().expect("Failed to create driver");

    let mut passed = 0;
    let mut total = 0;

    for case in CWE1234_CASES {
        total += 1;
        println!("\n--- Testing {}: {} ---", case.name, case.description);

        if let Err(e) = run_single_case(&driver, &config, case) {
            println!("FAIL: {}", e);
            panic!("Test failed for case '{}': {}", case.name, e);
        } else {
            passed += 1;
        }
    }

    assert_eq!(passed, total, "{} out of {} cases passed", passed, total);
    println!("\nAll {} CWE-1234 variant tests passed!", total);
}

/// Run a single test case and perform assertions
fn run_single_case(
    driver: &Driver,
    config: &Config,
    case: &CweTestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    let haystack_module = YosysModule::new(case.fixture_path, case.module_name)?;

    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    // let haystack_index = context.get(&haystack_key).unwrap().index();

    // Instantiate the query object first
    let query = UnlockLogic::<Search>::instantiate(Instance::root("unlock".to_string()));
    let results = query.query(driver, &context, &haystack_key, config);

    assert_eq!(
        results.len(),
        case.expected_matches,
        "Expected {} matches for {}, got {}",
        case.expected_matches,
        case.name,
        results.len()
    );

    // if let Some(min_depth) = case.min_or_depth {
    //     let depths: Vec<_> = results.iter().map(|r| r.or_tree_depth()).collect();
    //     let max_depth = depths.iter().max().copied().unwrap_or(0);
    //     assert!(
    //         max_depth >= min_depth,
    //         "Min OR tree depth {} not met for {} (max found: {})",
    //         min_depth,
    //         case.name,
    //         max_depth
    //     );
    // }

    // for (i, result) in results.iter().enumerate() {
    //     assert!(
    //         result.has_not_in_or_tree(haystack_index),
    //         "{} match {}: NOT must be in OR tree",
    //         case.name,
    //         i + 1
    //     );
    //     assert!(
    //         result.validate_connections(result.connections(), haystack_index),
    //         "{} match {}: Connections must be valid",
    //         case.name,
    //         i + 1
    //     );
    // }

    // if !results.is_empty() {
    //     let depths: Vec<_> = results.iter().map(|r| r.or_tree_depth()).collect();
    //     println!(
    //         "  Results: {} matches (depths: {:?})",
    //         results.len(),
    //         depths
    //     );
    // }

    Ok(())
}
