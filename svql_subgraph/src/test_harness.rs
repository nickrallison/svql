#![allow(dead_code)]

use std::sync::Once;

use svql_common::{Config, ConfigBuilder, YosysModule};

use crate::SubgraphMatcher;

static INIT: Once = Once::new();

/// Configures logging for the test runner.
pub fn setup_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

/// Specification for a subgraph matching integration test.
#[derive(Default)]
pub struct TestSpec<'a> {
    /// File path to the haystack netlist.
    pub haystack_path: &'a str,
    /// Name of the top module in the haystack.
    pub haystack_module: &'a str,
    /// File path to the pattern (needle) netlist.
    pub needle_path: &'a str,
    /// Name of the pattern module.
    pub needle_module: &'a str,
    /// Number of distinct matches expected.
    pub expected_count: usize,
    /// Optional configuration builder to override defaults
    pub config_fn: Option<fn(ConfigBuilder) -> ConfigBuilder>,
}

/// Run a query test using the new DataFrame API (ExecutionPlan + Store).
/// This uses the new `run_query` function which works for all pattern types.
#[track_caller]
pub fn run_query_test(spec: TestSpec) -> Result<(), Box<dyn std::error::Error>> {
    setup_test_logging();

    let mut config_builder = Config::builder();
    if let Some(f) = spec.config_fn {
        config_builder = f(config_builder);
    }
    let config = config_builder.build();

    // 1. Load the design
    let design_module: YosysModule = YosysModule::new(spec.haystack_path, spec.haystack_module)?;
    let needle_module: YosysModule = YosysModule::new(spec.needle_path, spec.needle_module)?;

    // 2. Import the design
    let design = design_module.import_design(&svql_common::ModuleConfig::default())?;
    let needle = needle_module.import_design(&svql_common::ModuleConfig::default())?;

    // Execute query using the new DataFrame API
    let assignment_set = SubgraphMatcher::enumerate_all(
        &needle,
        &design,
        needle_module.module_name().to_owned(),
        design_module.module_name().to_owned(),
        &config,
    );

    // Get the result count from the store

    if assignment_set.len() != spec.expected_count {
        tracing::error!(
            "Expected {} matches, found {} for needle {}\nhaystack {}",
            spec.expected_count,
            assignment_set.len(),
            spec.needle_module,
            spec.haystack_module
        );
        return Err(format!(
            "Expected {} matches, found {}",
            spec.expected_count,
            assignment_set.len()
        )
        .into());
    }

    Ok(())
}

/// Defines a test case for subgraph matching.
#[macro_export]
macro_rules! query_test {
    (
        name: $test_name:ident,
        needle: ($needle_path:expr, $needle_mod:expr),
        haystack: ($haystack_path:expr, $haystack_mod:expr),
        expect: $count:expr
    ) => {
        #[test]
        fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            $crate::test_harness::run_query_test($crate::test_harness::TestSpec {
                haystack_path: $haystack_path,
                haystack_module: $haystack_mod,
                needle_path: $needle_path,
                needle_module: $needle_mod,
                expected_count: $count,
                ..Default::default()
            })
        }
    };

    (
        name: $test_name:ident,
        needle: ($needle_path:expr, $needle_mod:expr),
        haystack: ($haystack_path:expr, $haystack_mod:expr),
        expect: $count:expr,
        config: $cfg_closure:expr
    ) => {
        #[test]
        fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            $crate::test_harness::run_query_test($crate::test_harness::TestSpec {
                haystack_path: $haystack_path,
                haystack_module: $haystack_mod,
                needle_path: $needle_path,
                needle_module: $needle_mod,
                expected_count: $count,
                config_fn: Some($cfg_closure),
            })
        }
    };
}
