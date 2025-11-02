// mod fsm_gap_tests;

use std::sync::OnceLock;
use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::cwe1245::Cwe1245;
use svql_query::traits::composite::SearchableComposite;
use svql_query::{Search, instance::Instance};

#[derive(Debug, Clone)]
struct Cwe1245TestCase {
    name: &'static str,
    fixture_path: &'static str,
    module_name: &'static str,
    description: &'static str,
    expected_matches: usize,
    #[allow(dead_code)]
    has_gaps: bool, // For validation
}

static CWE1245_CASES: &[Cwe1245TestCase] = &[
    Cwe1245TestCase {
        name: "simple_unreachable",
        fixture_path: "examples/fixtures/cwes/cwe1245/cwe1245_simple.v",
        module_name: "cwe1245_simple",
        description: "4-state FSM with unreachable state 3",
        expected_matches: 1,
        has_gaps: true,
    },
    Cwe1245TestCase {
        name: "fixed_complete",
        fixture_path: "examples/fixtures/cwes/cwe1245/cwe1245_fixed.v",
        module_name: "cwe1245_fixed",
        description: "Complete 4-state FSM (no gaps)",
        expected_matches: 0, // Filtered out
        has_gaps: false,
    },
    Cwe1245TestCase {
        name: "multi_deadlock",
        fixture_path: "examples/fixtures/cwes/cwe1245/cwe1245_multi.v",
        module_name: "cwe1245_multi",
        description: "Two FSMs: one complete, one deadlock",
        expected_matches: 1,
        has_gaps: true,
    },
    // Add more: e.g., incomplete_case (no default), binary_encoded.
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
fn test_cwe1245_all_variants() {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace().expect("Failed to create driver");

    let mut passed = 0;
    let mut total = 0;

    for case in CWE1245_CASES {
        total += 1;
        println!("\n--- Testing {}: {} ---", case.name, case.description);

        let haystack_module = YosysModule::new(case.fixture_path, case.module_name)?;
        let (haystack_key, haystack_design) = driver
            .get_or_load_design(
                &haystack_module.path().display().to_string(),
                haystack_module.module_name(),
                &config.haystack_options,
            )
            .unwrap();

        let context = Cwe1245::<Search>::context(&driver, &config.needle_options).unwrap();
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let results: Vec<_> = Cwe1245::<Search>::query(
            &haystack_key,
            &context,
            Instance::root("cwe1245".to_string()),
            &config,
        );

        assert_eq!(
            results.len(),
            case.expected_matches,
            "Expected {} matches for {}, got {}",
            case.expected_matches,
            case.name,
            results.len()
        );

        for result in &results {
            assert!(
                result.has_gaps(),
                "All matches must have gaps (unreachable/deadlock)"
            );
        }

        passed += 1;
        println!("  ✓ Passed ({} matches)", results.len());
    }

    assert_eq!(passed, total, "{} out of {} cases passed", passed, total);
    println!("\nAll {} CWE-1245 tests passed!", total);
}
