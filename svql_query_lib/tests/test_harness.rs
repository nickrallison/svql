#![allow(dead_code)]

use std::fmt::Debug;
use std::sync::{Arc, Once};
use svql_driver::design_container::DesignContainer;
use svql_query::{prelude::*, traits::Component};

static INIT: Once = Once::new();

pub fn setup_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

#[derive(Default)]
pub struct TestSpec<'a> {
    pub haystack_path: &'a str,
    pub haystack_module: &'a str,
    pub expected_count: usize,
    /// Optional configuration builder to override defaults
    pub config_fn: Option<fn(ConfigBuilder) -> ConfigBuilder>,
}

impl TestSpec<'_> {
    pub fn get_design(
        &self,
        driver: &Driver,
        config: &svql_common::Config,
    ) -> Result<Arc<DesignContainer>, QueryError> {
        let key = self.get_key();
        let container = driver
            .get_design(&key, &config.haystack_options)
            .map_err(|e| QueryError::design_load(e.to_string()))?;
        Ok(container)
    }

    pub fn get_key(&self) -> DriverKey {
        DriverKey::new(self.haystack_path, self.haystack_module)
    }
}

/// Run a query test using the new DataFrame API (ExecutionPlan + Store).
/// This uses the new `run_query` function which works for all pattern types.
#[track_caller]
pub fn run_query_test<P>(spec: TestSpec) -> Result<(), Box<dyn std::error::Error>>
where
    P: Pattern + Component + Send + Sync + Debug + 'static,
{
    setup_test_logging();

    let driver = Driver::new_workspace()?;

    let mut config_builder = Config::builder();
    if let Some(f) = spec.config_fn {
        config_builder = f(config_builder);
    }
    let config = config_builder.build();

    let _container = spec.get_design(&driver, &config)?;

    // for cell in container.index().cells_topo() {
    //     tracing::error!("Cell: {:#?}", cell);
    // }

    // Execute query using the new DataFrame API
    let store = svql_query::run_query::<P>(&driver, &spec.get_key(), &config)?;

    // Get the result count from the store
    let results_table = store.get::<P>().expect("Table should be present");
    let rows = results_table.rows().collect::<Vec<_>>();
    let stored_count = rows.len();

    if stored_count != spec.expected_count {
        tracing::error!(
            "Expected {} matches, found {} for pattern {}",
            spec.expected_count,
            stored_count,
            std::any::type_name::<P>()
        );
        let mut rehydrated: Vec<P> = Vec::new();
        for row in rows.iter() {
            let item = P::rehydrate(row, &store, &driver, &spec.get_key());
            if item.is_none() {
                tracing::error!("Failed to rehydrate row: {}", row);
                continue;
            }
            rehydrated.push(item.unwrap());
        }

        // let cells = container.index().cells_topo();
        tracing::error!(
            "Test Failed: Expected {} matches, found {}.\nQuery: {}\nHaystack: {} ({}), Store: {}",
            spec.expected_count,
            stored_count,
            std::any::type_name::<P>(),
            spec.haystack_module,
            spec.haystack_path,
            store
        );

        for (i, result) in rehydrated.iter().enumerate() {
            tracing::error!("Result #{}: {:#?}", i, result);
        }

        tracing::error!("Tables:");
        for (_, table) in store.tables() {
            tracing::error!("{}", table);
        }

        // let cells_str = cells
        //     .iter()
        //     .map(|c| format!(" - {:#?}", c))
        //     .collect::<Vec<String>>()
        //     .join("\n");

        // tracing::error!("Cell List: {}", cells_str);
        // Log match details if available
        if let Some(table) = store.get::<P>() {
            for (i, row) in table.rows().enumerate() {
                tracing::error!("Match #{}: {}", i, row);
            }
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
