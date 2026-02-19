use svql_query::{prelude::*, test_harness::TestSpec};
use svql_query_lib::LogicCone;

#[test]
fn test_logic_cone() -> Result<(), Box<dyn std::error::Error>> {
    let spec = TestSpec {
        haystack_module: "small_and_tree",
        haystack_path: "examples/fixtures/basic/and/verilog/small_and_tree.v",
        expected_count: 7,
        config_fn: None,
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();

    let driver = Driver::new_workspace()?;

    let mut config_builder = Config::builder();
    if let Some(f) = spec.config_fn {
        config_builder = f(config_builder);
    }
    let config = config_builder.build();

    // Execute query using the new DataFrame API
    let store = svql_query::run_query::<LogicCone>(&driver, &spec.get_key(), &config)?;

    // Get the result count from the store
    let results_table = store.get::<LogicCone>().expect("Table should be present");
    let rows = results_table.rows().collect::<Vec<_>>();

    // expects 1 logic cone of size 3 and 2 logic cones of size 1
    let mut expected_size = vec![(3, 1), (1, 2)];

    for (i, (_ref, row)) in rows.iter().enumerate() {
        let logic_cone =
            LogicCone::rehydrate(row, &store, &driver, &spec.get_key(), &config).unwrap();
        let size = logic_cone.size(&store, &driver, &spec.get_key());
        tracing::trace!("Row #{}: size = {}", i, size);

        // get the correct index for the actual size and decrement the count, removing it if it reaches zero

        if let Some((_, count)) = expected_size.iter_mut().find(|(s, _)| *s == size) {
            *count -= 1;
            if *count == 0 {
                expected_size.retain(|(s, _)| *s != size);
            }
        } else {
            panic!("Unexpected logic cone size: {}", size);
        }
    }

    if !expected_size.is_empty() {
        panic!(
            "Not all expected logic cone sizes were found. Remaining expected sizes: {:?}",
            expected_size
        );
    }

    // if stored_count != spec.expected_count {
    //     tracing::error!(
    //         "Expected {} matches, found {} for needle: {}, and haystack: {} ({}).",
    //         spec.expected_count,
    //         stored_count,
    //         std::any::type_name::<LogicCone>(),
    //         spec.haystack_module,
    //         spec.haystack_path
    //     );
    //     let mut rehydrated: Vec<LogicCone> = Vec::new();
    //     for row in rows.iter() {
    //         let item = LogicCone::rehydrate(row, &store, &driver, &spec.get_key());
    //         if item.is_none() {
    //             tracing::error!("Failed to rehydrate row: {}", row);
    //             continue;
    //         }
    //         rehydrated.push(item.unwrap());
    //     }

    //     // let cells = container.index().cells_topo();
    //     tracing::error!(
    //         "Test Failed: Expected {} matches, found {}.\nQuery: {}\nHaystack: {} ({}), Store: {}",
    //         spec.expected_count,
    //         stored_count,
    //         std::any::type_name::<LogicCone>(),
    //         spec.haystack_module,
    //         spec.haystack_path,
    //         store
    //     );

    //     for (i, result) in rehydrated.iter().enumerate() {
    //         tracing::trace!("Result #{}: {:#?}", i, result);
    //     }

    //     tracing::error!("Tables:");
    //     for (_, table) in store.tables() {
    //         tracing::error!("{}", table);
    //     }

    //     // let cells_str = cells
    //     //     .iter()
    //     //     .map(|c| format!(" - {:#?}", c))
    //     //     .collect::<Vec<String>>()
    //     //     .join("\n");

    //     // tracing::trace!("Cell List: {}", cells_str);
    //     if let Some(table) = store.get::<LogicCone>() {
    //         for (i, row) in table.rows().enumerate() {
    //             tracing::trace!("Match #{}: {}", i, row);
    //         }
    //     }
    // }

    // assert_eq!(
    //     stored_count,
    //     spec.expected_count,
    //     "Match count mismatch for {}",
    //     std::any::type_name::<LogicCone>()
    // );

    Ok(())
}
