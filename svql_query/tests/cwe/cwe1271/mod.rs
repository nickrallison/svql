use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::cwe1271::Cwe1271;
use svql_query::traits::{Query, Searchable};
use svql_query::{Search, instance::Instance};

#[derive(Debug, Clone)]
struct Cwe1271TestCase {
    name: &'static str,
    fixture_path: &'static str,
    module_name: &'static str,
    description: &'static str,
    expected_matches: usize,
}

static CWE1271_CASES: &[Cwe1271TestCase] = &[
    Cwe1271TestCase {
        name: "simple",
        fixture_path: "examples/fixtures/cwes/cwe1271/cwe1271_simple.v",
        module_name: "cwe1271_simple",
        description: "Basic uninitialized DFF (no enable, no reset)",
        expected_matches: 1,
    },
    Cwe1271TestCase {
        name: "en",
        fixture_path: "examples/fixtures/cwes/cwe1271/cwe1271_en.v",
        module_name: "cwe1271_en",
        description: "Uninitialized DFF with enable (no reset)",
        expected_matches: 2,
    },
    Cwe1271TestCase {
        name: "fixed",
        fixture_path: "examples/fixtures/cwes/cwe1271/cwe1271_fixed.v",
        module_name: "cwe1271_fixed",
        description: "FIXED: Initialized DFF with reset (secure)",
        expected_matches: 0,
    },
    Cwe1271TestCase {
        name: "multi",
        fixture_path: "examples/fixtures/cwes/cwe1271/cwe1271_multi.v",
        module_name: "cwe1271_multi",
        description: "Multiple uninitialized registers (2 UninitReg + 1 UninitRegEn)",
        expected_matches: 4,
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

fn run_single_case(
    driver: &Driver,
    config: &Config,
    case: &Cwe1271TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    let haystack_module = YosysModule::new(case.fixture_path, case.module_name)?;

    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = Cwe1271::<Search>::context(driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let query = Cwe1271::<Search>::instantiate(Instance::root("cwe1271".to_string()));
    let results = query.query(driver, &context, &haystack_key, config);

    assert_eq!(
        results.len(),
        case.expected_matches,
        "Case {}: expected {} matches, got {}",
        case.name,
        case.expected_matches,
        results.len()
    );

    Ok(())
}

#[test]
fn test_cwe1271_simple() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace()?;
    let case = CWE1271_CASES.iter().find(|c| c.name == "simple").unwrap();

    run_single_case(&driver, &config, case)
}

#[test]
fn test_cwe1271_en() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace()?;
    let case = CWE1271_CASES.iter().find(|c| c.name == "en").unwrap();

    run_single_case(&driver, &config, case)
}

#[test]
fn test_cwe1271_fixed() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace()?;
    let case = CWE1271_CASES.iter().find(|c| c.name == "fixed").unwrap();

    run_single_case(&driver, &config, case)
}

#[test]
fn test_cwe1271_multi() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace()?;
    let case = CWE1271_CASES.iter().find(|c| c.name == "multi").unwrap();

    run_single_case(&driver, &config, case)
}

#[test]
fn test_cwe1271_all_variants() {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace().expect("Failed to create driver");

    let mut passed = 0;
    let mut total = 0;

    println!("\n--- Running CWE-1271 Test Suite ---");

    for case in CWE1271_CASES {
        total += 1;
        println!("\nTesting {}: {}", case.name, case.description);

        if let Err(e) = run_single_case(&driver, &config, case) {
            println!("FAIL: {}", e);
            panic!("Test failed for case '{}': {}", case.name, e);
        } else {
            passed += 1;
            println!("  ✓ Passed");
        }
    }

    assert_eq!(passed, total, "{} out of {} cases passed", passed, total);
    println!("\nAll {} CWE-1271 tests passed!", total);
}
