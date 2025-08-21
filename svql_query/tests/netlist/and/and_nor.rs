// mod integration_tests {
//     #[cfg(test)]
//     mod and_nor {
//         use svql_driver::prelude::Driver;
//         use svql_driver::util::load_driver_from;
//         use svql_query::Search;
//         use svql_query::instance::Instance;
//         use svql_query::queries::netlist::basic::and::and_nor::AndNor;
//         use svql_subgraph::config::Config;

//         lazy_static::lazy_static! {
//             static ref MIXED_AND_TREE: Driver = load_driver_from("examples/fixtures/basic/and/json/mixed_and_tree.json").unwrap();
//             static ref AND_NOR: Driver = load_driver_from("examples/patterns/basic/and/verilog/and_nor.v").unwrap();
//             static ref CONFIG_DEDUPE_NONE: Config = Config::builder().exact_length().none().build();
//             static ref CONFIG_DEDUPE_AUTO_MORPH: Config = Config::builder().exact_length().auto_morph().build();
//         }

//         fn root_instance() -> Instance {
//             Instance::root("and_nor".to_string())
//         }

//         #[test]
//         fn and_nor_matches_in_mixed_tree_auto_morph() {
//             // mixed_and_tree has 2 and_nor instances
//             let hits = AndNor::<Search>::query(
//                 &*AND_NOR,
//                 &*MIXED_AND_TREE,
//                 root_instance(),
//                 &CONFIG_DEDUPE_AUTO_MORPH,
//             );
//             assert_eq!(
//                 hits.len(),
//                 2,
//                 "expected 2 and_nor matches in mixed_and_tree"
//             );
//         }

//         #[test]
//         fn and_nor_matches_in_mixed_tree_none() {
//             // mixed_and_tree has 2 and_nor instances
//             let hits = AndNor::<Search>::query(
//                 &*AND_NOR,
//                 &*MIXED_AND_TREE,
//                 root_instance(),
//                 &CONFIG_DEDUPE_NONE,
//             );
//             assert_eq!(
//                 hits.len(),
//                 4,
//                 "expected 4 and_nor matches in mixed_and_tree"
//             );
//         }
//     }
// }
