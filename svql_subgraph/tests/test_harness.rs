// use std::sync::Once;

// use svql_common::{Config, ConfigBuilder};
// use svql_driver::{Driver, DriverKey};

// static INIT: Once = Once::new();

// pub fn setup_test_logging() {
//     INIT.call_once(|| {
//         let _ = tracing_subscriber::fmt()
//             .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//             .with_test_writer()
//             .try_init();
//     });
// }

// pub struct TestSpec<'a> {
//     pub haystack_path: &'a str,
//     pub haystack_module: &'a str,
//     pub needle_path: &'a str,
//     pub needle_module: &'a str,
//     pub expected_count: usize,
//     /// Optional configuration builder to override defaults
//     pub config_fn: Option<fn(ConfigBuilder) -> ConfigBuilder>,
// }

// impl<'a> Default for TestSpec<'a> {
//     fn default() -> Self {
//         Self {
//             haystack_path: "",
//             haystack_module: "",
//             needle_path: "",
//             needle_module: "",
//             expected_count: 0,
//             config_fn: None,
//         }
//     }
// }

// /// Run a query test using the new DataFrame API (ExecutionPlan + Store).
// /// This uses the new `run_query` function which works for all pattern types.
// #[track_caller]
// pub fn run_query_test(spec: TestSpec) -> Result<(), Box<dyn std::error::Error>> {
//     setup_test_logging();

//     let driver = Driver::new_workspace()?;

//     let mut config_builder = Config::builder();
//     if let Some(f) = spec.config_fn {
//         config_builder = f(config_builder);
//     }
//     let config = config_builder.build();

//     let haystack_key: DriverKey = DriverKey::new(spec.haystack_path, spec.haystack_module);
//     let needle_key: DriverKey = DriverKey::new(spec.needle_path, spec.needle_module);

//     let design = driver.get_design(&haystack_key, &config.haystack_options)?;

//     // Execute query using the new DataFrame API
//     // let store = svql_q::run_query::<P>(&driver, &key)?;

//     // Get the result count from the store
//     let stored_count = store.get::<P>().map(|table| table.len()).unwrap_or(0);

//     if stored_count != spec.expected_count {
//         let cells = design.index().cells_topo();
//         tracing::error!(
//             "Test Failed: Expected {} matches, found {}.\nQuery: {}\nHaystack: {} ({}), Store: {}",
//             spec.expected_count,
//             stored_count,
//             std::any::type_name::<P>(),
//             spec.haystack_module,
//             spec.haystack_path,
//             store
//         );

//         tracing::error!("Tables:");
//         for (_, table) in store.tables() {
//             tracing::error!("{}", table);
//         }

//         let cells_str = cells
//             .iter()
//             .map(|c| format!(" - {:#?}", c))
//             .collect::<Vec<String>>()
//             .join("\n");

//         tracing::error!("Cell List: {}", cells_str);
//         // Log match details if available
//         if let Some(table) = store.get::<P>() {
//             for (i, row) in table.rows().enumerate() {
//                 tracing::error!("Match #{}: path={}", i, row.path());
//             }
//         }
//     }

//     assert_eq!(
//         stored_count,
//         spec.expected_count,
//         "Match count mismatch for {}",
//         std::any::type_name::<P>()
//     );

//     Ok(())
// }

// #[macro_export]
// macro_rules! query_test {
//     (
// 	        name: $test_name:ident,
// 	        query: $query_type:ty,
// 	        haystack: ($path:expr, $mod:expr),
// 	        expect: $count:expr
// 	    ) => {
//         #[test]
//         fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
//             $crate::test_harness::run_query_test::<$query_type>($crate::test_harness::TestSpec {
//                 haystack_path: $path,
//                 haystack_module: $mod,
//                 expected_count: $count,
//                 ..Default::default()
//             })
//         }
//     };

//     (
// 	        name: $test_name:ident,
// 	        query: $query_type:ty,
// 	        haystack: ($path:expr, $mod:expr),
// 	        expect: $count:expr,
// 	        config: $cfg_closure:expr
// 	    ) => {
//         #[test]
//         fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
//             $crate::test_harness::run_query_test::<$query_type>($crate::test_harness::TestSpec {
//                 haystack_path: $path,
//                 haystack_module: $mod,
//                 expected_count: $count,
//                 config_fn: Some($cfg_closure),
//             })
//         }
//     };
// }
