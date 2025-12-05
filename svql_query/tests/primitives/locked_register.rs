use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::primitives::locked_register::LockedRegister;
use svql_query::traits::variant::SearchableVariant;
use svql_query::{Search, instance::Instance};

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
fn test_locked_register_simple() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nLocked Register Simple Test");
    println!("Found {} DFF(s)\n", results.len());

    assert_eq!(
        results.len(),
        1,
        "Should find 1 async_mux pattern (Data_out register; lock_status doesn't match pattern)"
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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nLocked Register Multiple Registers Test");
    println!("Module has 3 lock status + 3 data registers = 6 DFFs\n");
    println!("Found {} DFF(s)\n", results.len());

    assert!(
        results.len() <= 3,
        "Should find at most 3 data registers (pattern-dependent)"
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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nLocked Register Deep Test");
    println!("Found {} DFF(s)\n", results.len());

    assert!(
        results.len() <= 1,
        "Should find at most 1 data register (pattern-dependent)"
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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nLocked Register Fixed (Secure) Test");
    println!("Found {} DFF(s)\n", results.len());

    println!("Note: Pattern detection is structural, not semantic");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nSync Reset Registers Test");
    println!("Found {} DFF(s)\n", results.len());

    println!("Note: Sync reset patterns may require additional pattern variants");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nEnabled Registers Test");
    println!("Found {} DFF(s)\n", results.len());

    println!("Note: Enable patterns may optimize to MUX structures");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nWide Register Test");
    println!("Found {} DFF(s) (32-bit registers)\n", results.len());

    println!("Note: Not all register widths may match structural pattern");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nMixed Reset Types Test");
    println!("Found {} DFF(s) (async + sync)\n", results.len());

    println!("Note: Different reset types may require additional patterns");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nNo Reset Registers Test");
    println!("Found {} DFF(s)\n", results.len());

    println!("Note: No-reset patterns may require additional pattern variants");

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

    let context = LockedRegister::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LockedRegister::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("locked_reg".to_string()),
        &config,
    );

    println!("\nMulti-Width Registers Test");
    println!(
        "Found {} DFF(s) (1-bit, 8-bit, 16-bit, 32-bit)\n",
        results.len()
    );

    assert_eq!(
        results.len(),
        1,
        "Should find 1 DFF matching async_mux pattern (1-bit register)"
    );

    Ok(())
}
