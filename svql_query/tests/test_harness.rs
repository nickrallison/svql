use std::sync::Once;
use svql_query::prelude::*;

// Ensure logging only initializes once across all tests
static INIT: Once = Once::new();

pub fn setup_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

/// Configuration for a specific test case
pub struct TestSpec<'a> {
    pub haystack_path: &'a str,
    pub haystack_module: &'a str,
    pub expected_count: usize,
    /// Optional configuration builder to override defaults
    pub config_fn: Option<fn(ConfigBuilder) -> ConfigBuilder>,
}

impl<'a> Default for TestSpec<'a> {
    fn default() -> Self {
        Self {
            haystack_path: "",
            haystack_module: "",
            expected_count: 0,
            config_fn: None,
        }
    }
}

/// The generic runner function.
pub fn run_query_test<P>(spec: TestSpec) -> Result<(), Box<dyn std::error::Error>>
where
    P: Pattern + 'static,
{
    setup_test_logging();

    let driver = Driver::new_workspace()?;

    let mut config_builder = Config::builder();
    if let Some(f) = spec.config_fn {
        config_builder = f(config_builder);
    }
    let config = config_builder.build();

    let (key, _) = driver.get_or_load_design(
        spec.haystack_path,
        spec.haystack_module,
        &config.haystack_options,
    )?;

    let matches = execute_query::<P>(&driver, &key, &config)?;

    if matches.len() != spec.expected_count {
        tracing::error!(
            "Test Failed: Expected {} matches, found {}.\nQuery: {}\nHaystack: {} ({})",
            spec.expected_count,
            matches.len(),
            std::any::type_name::<P>(),
            spec.haystack_module,
            spec.haystack_path
        );
        for (i, m) in matches.iter().enumerate() {
            let report = m.report(&format!("Match #{}", i));
            tracing::error!("{}", report.render());
        }
    }

    assert_eq!(
        matches.len(),
        spec.expected_count,
        "Match count mismatch for {}",
        std::any::type_name::<P>()
    );

    Ok(())
}

#[macro_export]
macro_rules! query_test {
    (
	        name: $test_name:ident,
	        query: $query_type:ty,
	        haystack: ($path:expr, $mod:expr),
	        expect: $count:expr
	    ) => {
        #[test]
        fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            $crate::test_harness::run_query_test::<$query_type>($crate::test_harness::TestSpec {
                haystack_path: $path,
                haystack_module: $mod,
                expected_count: $count,
                ..Default::default()
            })
        }
    };

    (
	        name: $test_name:ident,
	        query: $query_type:ty,
	        haystack: ($path:expr, $mod:expr),
	        expect: $count:expr,
	        config: $cfg_closure:expr
	    ) => {
        #[test]
        fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            $crate::test_harness::run_query_test::<$query_type>($crate::test_harness::TestSpec {
                haystack_path: $path,
                haystack_module: $mod,
                expected_count: $count,
                config_fn: Some($cfg_closure),
            })
        }
    };
}
