// mod integration_tests {
//     #[cfg(test)]
//     mod and {

//         use std::{path::PathBuf, sync::Arc};

//         use prjunnamed_netlist::Design;
//         use rstest::rstest;

//         lazy_static::lazy_static! {

//             static ref DRIVER: Driver = Driver::new_workspace().unwrap();

//             static ref AND_Q_DOUBLE_SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v", "and_q_double_sdffe".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v"));
//             static ref AND_GATE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/patterns/basic/and/verilog/and_gate.v", "and_gate".to_string()).unwrap(), PathBuf::from("examples/patterns/basic/and/verilog/and_gate.v"));
//             static ref AND_TREE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/and/verilog/and_tree.v", "and_tree".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/and/verilog/and_tree.v"));
//             static ref AND_SEQ: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/and/verilog/and_seq.v", "and_seq".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/and/verilog/and_seq.v"));

//             static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
//                 .match_length(false)
//                 .none()
//                 .build();
//         }

//         #[rstest]
//         // AND_Q_DOUBLE_SDFFE Needle
//         #[case(&AND_Q_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
//         #[case(&AND_Q_DOUBLE_SDFFE, &AND_GATE, 0)]
//         #[case(&AND_Q_DOUBLE_SDFFE, &AND_TREE, 0)]
//         #[case(&AND_Q_DOUBLE_SDFFE, &AND_SEQ, 0)]
//         // AND_GATE Needle
//         #[case(&AND_GATE, &AND_Q_DOUBLE_SDFFE, 1)]
//         #[case(&AND_GATE, &AND_GATE, 1)]
//         #[case(&AND_GATE, &AND_TREE, 7)]
//         #[case(&AND_GATE, &AND_SEQ, 7)]
//         // AND_TREE Needle
//         #[case(&AND_TREE, &AND_Q_DOUBLE_SDFFE, 0)]
//         #[case(&AND_TREE, &AND_GATE, 0)]
//         #[case(&AND_TREE, &AND_TREE, 1)]
//         #[case(&AND_TREE, &AND_SEQ, 0)]
//         // AND_SEQ Needle
//         #[case(&AND_SEQ, &AND_Q_DOUBLE_SDFFE, 0)]
//         #[case(&AND_SEQ, &AND_GATE, 0)]
//         #[case(&AND_SEQ, &AND_TREE, 0)]
//         #[case(&AND_SEQ, &AND_SEQ, 1)]
//         fn test_subgraph_matches(
//             #[case] needle: &(Arc<Design>, PathBuf),
//             #[case] haystack: &(Arc<Design>, PathBuf),
//             #[case] expected: usize,
//         ) {
//             let matches =
//                 svql_subgraph::find_subgraphs(needle.0.as_ref(), haystack.0.as_ref(), &CONFIG);
//             assert_eq!(
//                 matches.len(),
//                 expected,
//                 "Expected {} matches for needle {}, against haystack {}, got {}",
//                 expected,
//                 needle.1.display(),
//                 haystack.1.display(),
//                 matches.len()
//             );
//         }
//     }
// }
