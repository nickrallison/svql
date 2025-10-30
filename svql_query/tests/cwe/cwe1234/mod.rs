mod register;
mod unlock_logic;

use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::{
    Search, composite::SearchableComposite, instance::Instance, queries::security::cwe1234::Cwe1234,
};

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
fn test_cwe1234_simple_complete() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_simple.v",
        "cwe1234_simple",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = Cwe1234::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = Cwe1234::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("cwe1234".to_string()),
        &config,
    );

    println!("\n=== CWE-1234 Complete Pattern Test (Simple) ===");
    println!("Found {} complete vulnerability(ies)\n", results.len());

    assert_eq!(
        results.len(),
        1,
        "Should find NeedleSubsetHaystackly 1 complete CWE-1234 pattern"
    );

    Ok(())
}

#[test]
fn test_cwe1234_multi_reg_complete() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v",
        "cwe1234_multi_reg",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = Cwe1234::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = Cwe1234::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("cwe1234".to_string()),
        &config,
    );

    println!("\n=== CWE-1234 Multiple Registers Complete Test ===");
    println!("Module has 3 vulnerable data registers\n");
    println!("Found {} complete vulnerability(ies)\n", results.len());

    assert_eq!(
        results.len(),
        3,
        "Should find NeedleSubsetHaystackly 3 complete CWE-1234 patterns"
    );

    println!("\n✓ All 3 vulnerable data registers identified");
    println!("✓ Lock status registers correctly excluded");

    Ok(())
}

#[test]
fn test_cwe1234_fixed_complete() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_fixed.v",
        "cwe1234_fixed",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = Cwe1234::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = Cwe1234::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("cwe1234".to_string()),
        &config,
    );

    println!("\n=== CWE-1234 Fixed (Secure) Complete Test ===");
    println!("Found {} vulnerability(ies)\n", results.len());

    assert_eq!(
        results.len(),
        0,
        "Fixed version should have 0 complete CWE-1234 patterns"
    );

    println!("✓ No false positives - secure implementation validated");

    Ok(())
}
