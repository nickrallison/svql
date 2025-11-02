// svql_query/tests/primitives/logic_tree.rs

use std::sync::OnceLock;
use svql_common::{Config, Dedupe, MatchLength, YosysModule};
use svql_driver::Driver;
use svql_query::enum_composites::logic_tree::LogicTree;
use svql_query::traits::composite::SearchableComposite;
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
fn test_logic_tree_single_gate() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack = YosysModule::new(
        "examples/fixtures/composite/logic_tree/single_gate.v",
        "single_gate",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack.path().display().to_string(),
        haystack.module_name(),
        &config.haystack_options,
    )?;

    let context = LogicTree::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LogicTree::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("logic_tree".to_string()),
        &config,
    );

    println!("\n=== Single Gate Test ===");
    for tree in &results {
        println!("Tree: {:#?}", tree);
    }
    panic!();

    // // Should find at least the single AND gate as a leaf tree
    // assert!(
    //     !results.is_empty(),
    //     "Should find at least 1 tree (the AND gate)"
    // );

    // for (i, tree) in results.iter().take(5).enumerate() {
    //     println!("Tree {}: {}", i + 1, tree.describe());
    // }

    Ok(())
}

#[test]
fn test_logic_tree_simple_2level() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack = YosysModule::new(
        "examples/fixtures/composite/logic_tree/simple_2level.v",
        "simple_2level",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack.path().display().to_string(),
        haystack.module_name(),
        &config.haystack_options,
    )?;

    let context = LogicTree::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LogicTree::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("logic_tree".to_string()),
        &config,
    );

    println!("\n=== Simple 2-Level Test ===");
    for tree in &results {
        println!("Tree: {:#?}", tree);
    }
    panic!();
    // println!("Expected: AND of two ORs");
    // println!("Found {} tree(s)", results.len());

    // // Should find trees at multiple depths
    // let depths: Vec<_> = results.iter().map(|t| t.depth).collect();
    // let max_depth = depths.iter().max().copied().unwrap_or(0);

    // println!("Max depth found: {}", max_depth);
    // assert!(
    //     max_depth >= 2,
    //     "Should find depth 2 trees (AND with OR children)"
    // );

    // // Show some examples
    // for (i, tree) in results.iter().filter(|t| t.depth == 2).take(3).enumerate() {
    //     println!("Depth-2 tree {}: {}", i + 1, tree.describe());
    // }

    Ok(())
}

#[test]
fn test_logic_tree_deep_3level() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack = YosysModule::new(
        "examples/fixtures/composite/logic_tree/deep_3level.v",
        "deep_3level",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack.path().display().to_string(),
        haystack.module_name(),
        &config.haystack_options,
    )?;

    let context = LogicTree::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LogicTree::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("logic_tree".to_string()),
        &config,
    );

    println!("\n=== Deep 3-Level Test ===");
    for tree in &results {
        println!("Tree: {:#?}", tree);
    }
    panic!();
    // println!("Found {} tree(s)", results.len());

    // let depths: Vec<_> = results.iter().map(|t| t.depth).collect();
    // let max_depth = depths.iter().max().copied().unwrap_or(0);

    // println!("Max depth found: {}", max_depth);
    // assert!(max_depth >= 3, "Should find depth 3 trees");

    // // Show deepest examples
    // for (i, tree) in results.iter().filter(|t| t.depth >= 3).take(3).enumerate() {
    //     println!("Deep tree {}: {}", i + 1, tree.describe());
    // }

    Ok(())
}

#[test]
fn test_logic_tree_mixed_gates() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack = YosysModule::new(
        "examples/fixtures/composite/logic_tree/mixed_gates.v",
        "mixed_gates",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack.path().display().to_string(),
        haystack.module_name(),
        &config.haystack_options,
    )?;

    let context = LogicTree::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LogicTree::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("logic_tree".to_string()),
        &config,
    );

    println!("\n=== Mixed Gate Types Test ===");
    for tree in &results {
        println!("Tree: {:#?}", tree);
    }
    panic!();
    // println!("Found {} tree(s)", results.len());

    // // Verify we find trees with different gate types
    // for tree in &results {
    //     println!("Tree: {:?}", tree);
    // }

    // assert!(results.is_empty(), "Should find trees with mixed gates");

    Ok(())
}

#[test]
fn test_logic_tree_asymmetric() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    let config = Config::builder()
        .match_length(MatchLength::Exact)
        .dedupe(Dedupe::All)
        .build();

    let haystack = YosysModule::new(
        "examples/fixtures/composite/logic_tree/asymmetric_tree.v",
        "asymmetric_tree",
    )?;

    let driver = Driver::new_workspace()?;
    let (haystack_key, haystack_design) = driver.get_or_load_design(
        &haystack.path().display().to_string(),
        haystack.module_name(),
        &config.haystack_options,
    )?;

    let context = LogicTree::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    let results = LogicTree::<Search>::query(
        &haystack_key,
        &context,
        Instance::root("logic_tree".to_string()),
        &config,
    );

    println!("\n=== Asymmetric Tree Test ===");
    for tree in &results {
        println!("Tree: {:#?}", tree);
    }
    panic!();
    // println!("Found {} tree(s)", results.len());

    // let max_depth = results.iter().map(|t| t.depth).max().unwrap_or(0);
    // println!("Max depth: {}", max_depth);

    // assert!(
    //     max_depth >= 2,
    //     "Should find asymmetric trees with depth >= 2"
    // );

    Ok(())
}
