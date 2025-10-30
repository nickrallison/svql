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
        .match_length(MatchLength::Exact)
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
        "Should find exactly 1 complete CWE-1234 pattern"
    );

    Ok(())
}

#[test]
fn test_cwe1234_multi_reg_complete() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
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
        "Should find exactly 3 complete CWE-1234 patterns"
    );

    println!("\n✓ All 3 vulnerable data registers identified");
    println!("✓ Lock status registers correctly excluded");

    Ok(())
}

#[test]
fn test_cwe1234_fixed_complete() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
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

#[test]
fn test_cwe1234_all_variants_summary() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║      Complete CWE-1234 Vulnerability Detection Suite         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    let variants = vec![
        ("cwe1234_simple.v", "cwe1234_simple", "Simple pattern", 1),
        ("cwe1234_deep.v", "cwe1234_deep", "Deep OR tree", 1),
        (
            "cwe1234_multi_reg.v",
            "cwe1234_multi_reg",
            "Multiple regs",
            3,
        ),
        ("cwe1234_fixed.v", "cwe1234_fixed", "Fixed/Secure", 0),
        (
            "cwe1234_sync_reset.v",
            "cwe1234_sync_reset",
            "Sync reset",
            1,
        ),
        ("cwe1234_enabled.v", "cwe1234_enabled", "With enable", 1),
        ("cwe1234_wide_reg.v", "cwe1234_wide_reg", "Wide (32-bit)", 1),
    ];

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let driver = Driver::new_workspace()?;

    println!("┌────────────────────────────────┬──────────┬──────────┬────────┐");
    println!("│ Variant                        │ Expected │ Found    │ Status │");
    println!("├────────────────────────────────┼──────────┼──────────┼────────┤");

    let mut all_passed = true;

    for (filename, module_name, description, expected) in variants {
        let path = format!("examples/fixtures/cwes/cwe1234/{}", filename);
        let haystack_module = YosysModule::new(&path, module_name)?;

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

        let found = results.len();
        let status = if found == expected {
            "✓ PASS"
        } else {
            "✗ FAIL"
        };

        if found != expected {
            all_passed = false;
        }

        println!(
            "│ {:<30} │ {:>8} │ {:>8} │ {:<6} │",
            description, expected, found, status
        );
    }

    println!("└────────────────────────────────┴──────────┴──────────┴────────┘");
    println!();

    assert!(
        all_passed,
        "Some tests did not find expected vulnerabilities"
    );

    println!("✓ All complete CWE-1234 tests passed!");
    println!("\nComplete pattern validation:");
    println!("  - Unlock logic: AND + OR tree + negated lock");
    println!("  - Locked register: DFF storing protected data");
    println!("  - Connection: unlock output → register enable");
    println!("\nKey features:");
    println!("  ✓ Excludes lock_status DFFs (not vulnerable)");
    println!("  ✓ Identifies data DFFs with bypass logic (vulnerable)");
    println!("  ✓ No false positives on secure implementations");

    Ok(())
}
