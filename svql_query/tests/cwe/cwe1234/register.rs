use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::{
    Search,
    composite::{SearchableComposite, SearchableEnumComposite},
    instance::Instance,
    queries::security::cwe1234::register::RegisterAny,
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

// ============================================================================
// Basic DFF Detection Tests
// ============================================================================

#[test]
fn test_locked_register_simple() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Locked Register Simple Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // cwe1234_simple.v has 2 DFFs: lock_status and Data_out
    assert_eq!(
        results.len(),
        2,
        "Should find exactly 2 DFFs in simple design (lock_status + Data_out)"
    );

    Ok(())
}

#[test]
fn test_locked_register_multi_reg() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Locked Register Multiple Registers Test ===");
    println!("Module has 3 lock status + 3 data registers = 6 DFFs\n");
    println!("Found {} DFF(s)\n", results.len());

    // cwe1234_multi_reg.v has 3 lock_status DFFs + 3 Data_out DFFs = 6 total
    assert_eq!(
        results.len(),
        6,
        "Should find exactly 6 DFFs (3 lock_status + 3 Data_out)"
    );

    Ok(())
}

#[test]
fn test_locked_register_deep() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_deep.v",
        "cwe1234_deep",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Locked Register Deep Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // cwe1234_deep.v has 1 lock_status + 1 Data_out = 2 total
    assert_eq!(
        results.len(),
        2,
        "Should find exactly 2 DFFs (lock_status + Data_out)"
    );

    Ok(())
}

#[test]
fn test_locked_register_fixed() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Locked Register Fixed (Secure) Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // cwe1234_fixed.v also has 2 DFFs, but they're used securely
    assert_eq!(
        results.len(),
        2,
        "Should find exactly 2 DFFs (lock_status + Data_out)"
    );

    Ok(())
}
#[test]
fn test_locked_register_sync_reset() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v",
        "cwe1234_sync_reset",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Sync Reset Registers Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // Should find 2 DFFs (lock_status + Data_out) with synchronous reset
    assert_eq!(results.len(), 2, "Should find 2 sync reset DFFs");

    Ok(())
}

#[test]
fn test_locked_register_enabled() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_enabled.v",
        "cwe1234_enabled",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Enabled Registers Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // Should find 2 DFFs with explicit enable signals
    assert_eq!(results.len(), 2, "Should find 2 enabled DFFs");

    Ok(())
}

#[test]
fn test_locked_register_wide() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v",
        "cwe1234_wide_reg",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Wide Register Test ===");
    println!("Found {} DFF(s) (32-bit registers)\n", results.len());

    // Should find 2 DFFs (lock_status is 1-bit, Data_out is 32-bit)
    // Yosys typically creates 1 DFF cell for lock, 1 for the 32-bit data
    assert_eq!(
        results.len(),
        2,
        "Should find 2 DFFs (1-bit lock + 32-bit data)"
    );

    Ok(())
}

#[test]
fn test_locked_register_mixed_resets() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v",
        "cwe1234_mixed_resets",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Mixed Reset Types Test ===");
    println!("Found {} DFF(s) (async + sync)\n", results.len());

    // Should find 4 DFFs: 2 lock_status (1 async, 1 sync) + 2 Data_out (1 async, 1 sync)
    assert_eq!(results.len(), 4, "Should find 4 DFFs (2 async + 2 sync)");

    Ok(())
}

#[test]
fn test_locked_register_no_reset() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v",
        "cwe1234_no_reset",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== No Reset Registers Test ===");
    println!("Found {} DFF(s)\n", results.len());

    // Should find 2 DFFs without reset
    assert_eq!(results.len(), 2, "Should find 2 DFFs without reset");

    Ok(())
}

#[test]
fn test_locked_register_multi_width() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v",
        "cwe1234_multi_width",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = RegisterAny::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\n=== Multi-Width Registers Test ===");
    println!(
        "Found {} DFF(s) (1-bit, 8-bit, 16-bit, 32-bit)\n",
        results.len()
    );

    // Should find 5 DFFs: 1 lock + 4 data registers (different widths)
    assert_eq!(
        results.len(),
        5,
        "Should find 5 DFFs (1 lock + 4 data of varying widths)"
    );

    Ok(())
}

// Update the summary test to include new variants
#[test]
fn test_locked_register_all_variants_summary() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    println!("\n");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        Locked Register Detection Test Suite               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let variants = vec![
        ("cwe1234_simple.v", "cwe1234_simple", "Simple (async)", 2),
        (
            "cwe1234_sync_reset.v",
            "cwe1234_sync_reset",
            "Sync reset",
            2,
        ),
        ("cwe1234_enabled.v", "cwe1234_enabled", "With enable", 2),
        ("cwe1234_wide_reg.v", "cwe1234_wide_reg", "Wide (32-bit)", 2),
        (
            "cwe1234_mixed_resets.v",
            "cwe1234_mixed_resets",
            "Mixed resets",
            4,
        ),
        ("cwe1234_no_reset.v", "cwe1234_no_reset", "No reset", 2),
        (
            "cwe1234_multi_width.v",
            "cwe1234_multi_width",
            "Multi-width",
            5,
        ),
        ("cwe1234_deep.v", "cwe1234_deep", "Deep OR tree", 2),
        (
            "cwe1234_multi_reg.v",
            "cwe1234_multi_reg",
            "Multiple (6)",
            6,
        ),
        ("cwe1234_fixed.v", "cwe1234_fixed", "Fixed/Secure", 2),
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

        let context = RegisterAny::<Search>::context(&driver, &config.needle_options)?;
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let results = RegisterAny::<Search>::query(
            &haystack_key,
            &context,
            Instance::root("locked_reg".to_string()),
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
        "Some tests did not find expected exact number of DFFs"
    );

    println!("✓ All locked register tests passed!");
    println!("\nRegister types tested:");
    println!("  - Async reset DFFs");
    println!("  - Sync reset DFFs");
    println!("  - Enabled DFFs");
    println!("  - Wide registers (32-bit)");
    println!("  - Mixed reset types");
    println!("  - No reset DFFs");
    println!("  - Multi-width registers (1/8/16/32-bit)");

    Ok(())
}
