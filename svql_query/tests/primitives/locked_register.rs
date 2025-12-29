use std::sync::OnceLock;
use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::primitives::locked_register::LockedRegister;
use svql_query::traits::{Query, Searchable};
use svql_query::{Search, instance::Instance};

struct LockedRegisterTestCase {
    name: &'static str,
    fixture_path: &'static str,
    module_name: &'static str,
    expected_matches: usize,
    match_length: MatchLength,
}

static LOCKED_REGISTER_CASES: &[LockedRegisterTestCase] = &[
    LockedRegisterTestCase {
        name: "simple",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_simple.v",
        module_name: "cwe1234_simple",
        expected_matches: 1,
        match_length: MatchLength::NeedleSubsetHaystack,
    },
    LockedRegisterTestCase {
        name: "multi_reg",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v",
        module_name: "cwe1234_multi_reg",
        expected_matches: 3,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "deep",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_deep.v",
        module_name: "cwe1234_deep",
        expected_matches: 1,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "fixed",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_fixed.v",
        module_name: "cwe1234_fixed",
        expected_matches: 0,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "sync_reset",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v",
        module_name: "cwe1234_sync_reset",
        expected_matches: 1,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "enabled",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_enabled.v",
        module_name: "cwe1234_enabled",
        expected_matches: 1,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "wide",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v",
        module_name: "cwe1234_wide_reg",
        expected_matches: 1,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "mixed_resets",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v",
        module_name: "cwe1234_mixed_resets",
        expected_matches: 2,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "no_reset",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v",
        module_name: "cwe1234_no_reset",
        expected_matches: 1,
        match_length: MatchLength::Exact,
    },
    LockedRegisterTestCase {
        name: "multi_width",
        fixture_path: "examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v",
        module_name: "cwe1234_multi_width",
        expected_matches: 1,
        match_length: MatchLength::Exact,
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

fn run_locked_reg_case(
    driver: &Driver,
    case: &LockedRegisterTestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .match_length(case.match_length.clone())
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(case.fixture_path, case.module_name)?;

    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = LockedRegister::<Search>::context(driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let query = LockedRegister::<Search>::instantiate(Instance::root("locked_reg".to_string()));
    let results = query.query(driver, &context, &haystack_key, &config);

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
fn test_locked_register_primitives() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();
    let driver = Driver::new_workspace()?;

    for case in LOCKED_REGISTER_CASES {
        run_locked_reg_case(&driver, case)?;
    }

    Ok(())
}
