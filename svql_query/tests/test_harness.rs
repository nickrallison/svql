use std::sync::Once;
use svql_query::prelude::*;

static INIT: Once = Once::new();

pub fn setup_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

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

#[track_caller]
pub fn run_query_test<P>(spec: TestSpec) -> Result<(), Box<dyn std::error::Error>>
where
    P: Pattern + 'static,
    P::Match: Dehydrate + Rehydrate,
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

    // Execute query directly into session (dehydrated storage)
    let session = execute_query_session::<P>(&driver, &key, &config)?;

    // Verify dehydrated count matches
    let stored_count = session
        .results()
        .get_by_name(<P::Match as Dehydrate>::SCHEMA.type_name)
        .map(|r| r.len())
        .unwrap_or(0);

    if stored_count != spec.expected_count {
        // Rehydrate for error reporting
        let ctx = session.rehydrate_context();
        let matches: Vec<P::Match> = RehydrateIter::new(&ctx)
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        
        tracing::error!(
            "Test Failed: Expected {} matches, found {}.\nQuery: {}\nHaystack: {} ({})",
            spec.expected_count,
            stored_count,
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
        stored_count,
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
