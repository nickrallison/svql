use std::sync::OnceLock;
use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::cwe1262::Cwe1262;
use svql_query::{Search, instance::Instance};
use svql_query::traits::composite::SearchableComposite;

#[derive(Debug, Clone)]
struct Cwe1262TestCase {
    name: &'static str,
    fixture_path: &'static str,
    module_name: &'static str,
    description: &'static str,
    expected_matches: usize,
}

static CWE1262_CASES: &[Cwe1262TestCase] = &[
    Cwe1262TestCase {
        name: "uniform_secure",
        fixture_path: "examples/fixtures/cwes/cwe1262/cwe1262_uniform.v",
        module_name: "cwe1262_uniform",
        description: "Uniform lock across array (secure)",
        expected_matches: 0,
    },
    Cwe1262TestCase {
        name: "inconsistent_vuln",
        fixture_path: "examples/fixtures/cwes/cwe1262/cwe1262_inconsistent.v",
        module_name: "cwe1262_inconsistent",
        description: "Varying locks in array (vulnerable)",
        expected_matches: 1,
    },
    Cwe1262TestCase {
        name: "multi_arrays",
        fixture_path: "examples/fixtures/cwes/cwe1262/cwe1262_multi.v",
        module_name: "cwe1262_multi",
        description: "One uniform, one inconsistent array",
        expected_matches: 1,
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
fn test_cwe1262_all_variants() {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace().expect("Failed to create driver");

    let mut passed = 0;
    let mut total = 0;

    for case in CWE1262_CASES {
        total += 1;
        println!("\n--- Testing {}: {} ---", case.name, case.description);

        let haystack_module = YosysModule::new(case.fixture_path, case.module_name).unwrap();
        let (haystack_key, haystack_design) = driver.get_or_load_design(
            &haystack_module.path().display().to_string(),
            haystack_module.module_name(),
            &config.haystack_options,
        ).unwrap();

        let context = Cwe1262::<Search>::context(&driver, &config.needle_options).unwrap();
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let results: Vec<_> = Cwe1262::<Search>::query(
            &haystack_key,
            &context,
            Instance::root("cwe1262".to_string()),
            &config,
        );

        assert_eq!(
            results.len(),
            case.expected_matches,
            "Expected {} matches for {}, got {}",
            case.expected_matches, case.name, results.len()
        );

        for result in &results {
            assert!(result.is_inconsistent(), "All matches must be inconsistent");
        }

        passed += 1;
        println!("  ✓ Passed ({} matches)", results.len());
    }

    assert_eq!(passed, total, "{} out of {} cases passed", passed, total);
    println!("\nAll {} CWE-1262 tests passed!", total);
}
