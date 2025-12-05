use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::security::cwe1234::unlock_logic::UnlockLogic;
use svql_query::traits::composite::{Composite, MatchedComposite};
use svql_query::{Search, instance::Instance, traits::composite::SearchableComposite};

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
fn test_cwe1234_simple() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Simple Pattern Test");
    println!("Pattern: write & (~lock | debug)");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        1,
        "Should find exactly 1 minimal pattern: write & (~lock | debug)"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {}:", i + 1);
        println!("  - OR depth: {}", result.or_tree_depth());
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "NOT must be in OR tree"
        );
        assert!(
            result.validate_connections(result.connections(), haystack_index),
            "Connections must be valid"
        );
        println!("  ✓ Valid simple bypass pattern\n");
    }

    Ok(())
}

#[test]
fn test_cwe1234_deep() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Deep OR Tree Test");
    println!("Pattern: write & (((~lock | scan) | debug) | test_mode)");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        1,
        "Should find exactly 1 deep OR tree pattern with 4 bypass conditions"
    );

    let max_depth = results.iter().map(|r| r.or_tree_depth()).max().unwrap_or(0);
    println!("Maximum OR tree depth: {}", max_depth);

    assert!(
        max_depth >= 2,
        "Should have depth >= 2 for deep tree, found {}",
        max_depth
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {} (depth {}):", i + 1, result.or_tree_depth());
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "NOT must be in deep OR tree"
        );
        println!("  ✓ Valid deep tree pattern\n");
    }

    Ok(())
}

#[test]
fn test_cwe1234_swapped() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_swapped.v",
        "cwe1234_swapped",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Swapped Inputs Test");
    println!("Pattern: (scan | ~lock | debug) & write");
    println!("Tests commutative AND matching\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        1,
        "Should find exactly 1 pattern even with swapped AND inputs"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {}:", i + 1);
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "NOT must be in OR tree"
        );
        println!("  ✓ Valid swapped input pattern\n");
    }

    println!("✓ Handles commutative AND gate inputs");

    Ok(())
}

#[test]
fn test_cwe1234_combined() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_combined.v",
        "cwe1234_combined",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Combined Logic Test");
    println!("Pattern: write & ((~lock & mode_a) | debug)");
    println!("Tests complex AND/OR combinations\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        0,
        "Current pattern does not match AND within OR tree structure"
    );

    Ok(())
}

#[test]
fn test_cwe1234_multi_reg() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Multiple Registers Test");
    println!("Module has 3 vulnerable registers:");
    println!("  1. write_1 & (~lock_1 | scan)");
    println!("  2. write_2 & (~lock_2 | debug)");
    println!("  3. write_3 & (~lock_3 | scan | debug | test)\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        3,
        "Should find exactly 3 vulnerable registers"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {} (depth {}):", i + 1, result.or_tree_depth());
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "Each must have NOT in OR tree"
        );
        println!("  ✓ Valid vulnerability\n");
    }

    println!("✓ All 3 vulnerable registers detected");

    Ok(())
}

#[test]
fn test_cwe1234_not_positions() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_not_positions.v",
        "cwe1234_not_positions",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 NOT Position Variants Test");
    println!("Tests NOT at different horizontal positions:");
    println!("  1. (~lock | scan | debug)    - NOT leftmost");
    println!("  2. (scan | ~lock | debug)    - NOT middle");
    println!("  3. (scan | debug | ~lock)    - NOT rightmost");
    println!("  4. ((~lock | scan) | ...)    - NOT nested\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        5,
        "Nested pattern creates multiple valid matches at different tree levels"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {} (depth {}):", i + 1, result.or_tree_depth());
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "Must find NOT regardless of position"
        );
        println!("  ✓ NOT found at position {}\n", i + 1);
    }

    println!("✓ All horizontal positions detected (including nested)");

    Ok(())
}

#[test]
fn test_cwe1234_not_deep() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_not_deep.v",
        "cwe1234_not_deep",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 NOT Depth Variants Test");
    println!("Tests NOT at different vertical depths:");
    println!("  Depth 1: OR(~lock, bypass)");
    println!("  Depth 2: OR(OR(~lock, b1), b2)");
    println!("  Depth 3: OR(OR(OR(~lock, b1), b2), ...)\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        3,
        "Should find exactly 3 patterns at different depths (one per register)"
    );

    let depths: Vec<usize> = results.iter().map(|r| r.or_tree_depth()).collect();
    let max_depth = *depths.iter().max().unwrap();

    println!("Depth distribution:");
    for depth in 1..=max_depth {
        let count = depths.iter().filter(|&&d| d == depth).count();
        if count > 0 {
            println!("  Depth {}: {} match(es)", depth, count);
        }
    }

    assert!(
        max_depth >= 3,
        "Should find NOT at depth 3+, max found: {}",
        max_depth
    );

    for (i, result) in results.iter().enumerate() {
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "Match {}: NOT must be found at depth {}",
            i + 1,
            result.or_tree_depth()
        );
    }

    println!("\n✓ Recursive traversal works at all 3 depths");

    Ok(())
}

#[test]
fn test_cwe1234_not_right() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_not_right.v",
        "cwe1234_not_right",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 NOT on Right Input Test");
    println!("Tests NOT consistently on right side of OR gates:");
    println!("  1. OR(bypass, ~lock)");
    println!("  2. OR(a, OR(b, ~lock))");
    println!("  3. OR(..., OR(c, ~lock))\n");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        3,
        "Should find exactly 3 right-side patterns (one per register)"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {} (depth {}):", i + 1, result.or_tree_depth());
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "Must find NOT on right input"
        );
        println!("  ✓ NOT found on right side\n");
    }

    println!("✓ Both OR inputs (a and b) correctly checked");

    Ok(())
}

#[test]
fn test_cwe1234_not_alternating() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v",
        "cwe1234_not_alternating",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let haystack_index = context.get(&haystack_key).unwrap().index();

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Alternating Position Test");
    println!("Tests complex zigzag patterns:");
    println!("  Pattern 1: write & (~lock | (bypass_a | bypass_b))");
    println!("  Pattern 2: ((bypass_a | bypass_b) | ~lock) & write\n");
    println!("Found {} match(es)\n", results.len());
    assert_eq!(
        results.len(),
        4,
        "Alternating patterns match at multiple tree levels"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Match {}:", i + 1);
        assert!(
            result.has_not_in_or_tree(haystack_index),
            "Must find NOT in alternating structure"
        );
        println!("  ✓ Valid alternating pattern\n");
    }

    println!("✓ Handles complex alternating structures (with multi-level matches)");

    Ok(())
}

#[test]
fn test_cwe1234_fixed_no_vulnerability() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    println!("\nCWE1234 Fixed (No Vulnerability) Test");
    println!("Pattern: write & ~lock  (NO bypass conditions)");
    println!("Found {} match(es)\n", results.len());

    assert_eq!(
        results.len(),
        0,
        "FIXED version should have exactly 0 vulnerabilities (no bypass)"
    );

    println!("✓ No false positives - correctly identifies secure code");

    Ok(())
}
