// svql_query/tests/security_tests.rs

use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::{
    Search,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
    queries::security::cwe1234::unlock_logic::UnlockLogic,
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
fn test_unlock_logic_cwe1234() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    // Load the CWE1234 example haystack
    let haystack_module = YosysModule::new(
        "examples/fixtures/security/access_control/locked_reg/verilog/cwe1234.v",
        "cwe1234",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack_module.path().display().to_string(),
        haystack_module.module_name(),
        &config.haystack_options,
    )?;

    // Build context for UnlockLogic query
    let context = UnlockLogic::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    // Query for unlock patterns
    let results = UnlockLogic::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("unlock".to_string()),
        &config,
    );

    // Assertions
    assert!(
        !results.is_empty(),
        "Should find at least one unlock pattern in CWE1234 example (write & (~lock_status | scan_mode | debug_unlocked))"
    );

    println!("\n=== CWE1234 Unlock Pattern Analysis ===");
    println!("Found {} unlock pattern(s)\n", results.len());

    // Verify each match has the expected structure
    for (i, result) in results.iter().enumerate() {
        println!("Match {}:", i + 1);
        println!("  - Top AND gate: {}", result.top_and.path.inst_path());
        println!("  - OR tree depth: {}", result.or_tree_depth());
        println!("  - OR tree path: {}", result.rec_or.path.inst_path());
        println!("  - NOT gate path: {}", result.not_gate.path.inst_path());

        // Validate structure
        assert!(
            result.top_and.path.inst_path().contains("top_and"),
            "Match {} should have top_and gate",
            i + 1
        );

        assert!(
            result.rec_or.depth() >= 1,
            "Match {} should have at least depth-1 OR tree (found depth {})",
            i + 1,
            result.rec_or.depth()
        );

        // Critical validation: NOT gate must be in the OR tree
        assert!(
            result.has_not_in_or_tree(),
            "Match {} MUST have NOT gate feeding into OR tree (this is the vulnerability!)",
            i + 1
        );

        // Validate the top-level connection (OR -> AND)
        assert!(
            result.validate_connections(result.connections()),
            "Match {} should have valid OR->AND connection",
            i + 1
        );

        println!("  ✓ All validations passed\n");
    }

    println!("=== Expected Pattern ===");
    println!("The CWE1234 vulnerability pattern is:");
    println!("  write & (~lock_status | scan_mode | debug_unlocked)");
    println!("       ^         ^             ^            ^");
    println!("       |         |             |            |");
    println!("    top_and   not_gate    bypass_1     bypass_2");
    println!("                 |             |            |");
    println!("                 +-------------+------------+");
    println!("                              |");
    println!("                           rec_or (OR tree)");
    println!("\nThis allows bypassing lock protection via debug modes!\n");

    Ok(())
}

#[test]
fn test_unlock_logic_negative_case() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    // Use a design without the unlock pattern (simple AND tree)
    let haystack_module = YosysModule::new(
        "examples/fixtures/basic/and/verilog/small_and_tree.v",
        "small_and_tree",
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

    // Should find 0 matches - this design doesn't have the vulnerability pattern
    assert_eq!(
        results.len(),
        0,
        "Should NOT find unlock pattern in simple AND tree (no NOT->OR->AND structure)"
    );

    println!("\n✓ Negative test passed: No false positives in simple AND tree");

    Ok(())
}

#[test]
fn test_unlock_logic_depth_variations() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/security/access_control/locked_reg/verilog/cwe1234.v",
        "cwe1234",
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

    println!("\n=== OR Tree Depth Analysis ===");

    // Collect depth statistics
    let depths: Vec<usize> = results.iter().map(|r| r.or_tree_depth()).collect();

    if !depths.is_empty() {
        let min_depth = *depths.iter().min().unwrap();
        let max_depth = *depths.iter().max().unwrap();

        println!("Min OR tree depth: {}", min_depth);
        println!("Max OR tree depth: {}", max_depth);

        // The CWE1234 example has: ~lock_status | scan_mode | debug_unlocked
        // This could be represented as:
        // - Depth 1: single OR with all inputs
        // - Depth 2: (a | b) | c structure
        assert!(min_depth >= 1, "Should have at least depth-1 OR tree");

        // Group by depth
        for depth in min_depth..=max_depth {
            let count = depths.iter().filter(|&&d| d == depth).count();
            println!("Depth {}: {} match(es)", depth, count);
        }
    }

    Ok(())
}

#[rstest]
fn test_unlock_logic_all_components_required() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack_module = YosysModule::new(
        "examples/fixtures/security/access_control/locked_reg/verilog/cwe1234.v",
        "cwe1234",
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

    println!("\n=== Component Validation ===");

    for (i, result) in results.iter().enumerate() {
        println!("\nMatch {}: Component Check", i + 1);

        // 1. Must have AND gate
        let has_and = !result.top_and.path.inst_path().is_empty();
        println!("  ✓ Has top-level AND gate: {}", has_and);
        assert!(has_and, "Must have top-level AND gate");

        // 2. Must have OR tree
        let has_or_tree = result.rec_or.depth() >= 1;
        println!(
            "  ✓ Has OR tree (depth={}): {}",
            result.rec_or.depth(),
            has_or_tree
        );
        assert!(has_or_tree, "Must have OR tree");

        // 3. Must have NOT gate
        let has_not = !result.not_gate.path.inst_path().is_empty();
        println!("  ✓ Has NOT gate: {}", has_not);
        assert!(has_not, "Must have NOT gate");

        // 4. Critical: NOT must connect to OR tree
        let not_in_tree = result.has_not_in_or_tree();
        println!("  ✓ NOT connects to OR tree: {}", not_in_tree);
        assert!(
            not_in_tree,
            "NOT gate MUST connect to OR tree (this is the CWE1234 vulnerability signature)"
        );

        // 5. OR must connect to AND
        let or_to_and = result.validate_connections(result.connections());
        println!("  ✓ OR connects to AND: {}", or_to_and);
        assert!(or_to_and, "OR tree must connect to AND gate");
    }

    println!("\n✓ All components present and properly connected");

    Ok(())
}
