use crate::driver::net::SvqlDriverNetError;
use crate::driver::DriverIterator;
use std::collections::HashMap;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::matches::{IdString, SanitizedCellData, SanitizedQueryMatch};

pub struct MockDriver;

impl MockDriver {
    pub fn new() -> Self {
        MockDriver
    }

    pub fn query(&self, _cfg: &SvqlRuntimeConfig) -> Result<DriverIterator, SvqlDriverNetError> {
        // Return the three matches from the provided output
        let matches = vec![
            // Match 0
            SanitizedQueryMatch {
                port_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        IdString::Named("b".to_string()),
                        IdString::Named("d".to_string()),
                    );
                    map.insert(
                        IdString::Named("y".to_string()),
                        IdString::Unnamed {
                            gate_name: "and".to_string(),
                            file_path: "svql_query/verilog/many_ands.v".to_string(),
                            line: "16".to_string(),
                            id: "3_Y".to_string(),
                        },
                    );
                    map.insert(
                        IdString::Named("a".to_string()),
                        IdString::Unnamed {
                            gate_name: "and".to_string(),
                            file_path: "svql_query/verilog/many_ands.v".to_string(),
                            line: "16".to_string(),
                            id: "2_Y".to_string(),
                        },
                    );
                    map
                },
                cell_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/and.v".to_string(),
                                line: "9".to_string(),
                                id: "41".to_string(),
                            },
                            cell_index: 553,
                        },
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/many_ands.v".to_string(),
                                line: "16".to_string(),
                                id: "3".to_string(),
                            },
                            cell_index: 550,
                        },
                    );
                    map
                },
            },
            // Match 1
            SanitizedQueryMatch {
                port_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        IdString::Named("a".to_string()),
                        IdString::Unnamed {
                            gate_name: "and".to_string(),
                            file_path: "svql_query/verilog/many_ands.v".to_string(),
                            line: "16".to_string(),
                            id: "1_Y".to_string(),
                        },
                    );
                    map.insert(
                        IdString::Named("b".to_string()),
                        IdString::Named("c".to_string()),
                    );
                    map.insert(
                        IdString::Named("y".to_string()),
                        IdString::Unnamed {
                            gate_name: "and".to_string(),
                            file_path: "svql_query/verilog/many_ands.v".to_string(),
                            line: "16".to_string(),
                            id: "2_Y".to_string(),
                        },
                    );
                    map
                },
                cell_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/and.v".to_string(),
                                line: "9".to_string(),
                                id: "41".to_string(),
                            },
                            cell_index: 553,
                        },
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/many_ands.v".to_string(),
                                line: "16".to_string(),
                                id: "2".to_string(),
                            },
                            cell_index: 547,
                        },
                    );
                    map
                },
            },
            // Match 2
            SanitizedQueryMatch {
                port_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        IdString::Named("y".to_string()),
                        IdString::Unnamed {
                            gate_name: "and".to_string(),
                            file_path: "svql_query/verilog/many_ands.v".to_string(),
                            line: "16".to_string(),
                            id: "1_Y".to_string(),
                        },
                    );
                    map.insert(
                        IdString::Named("a".to_string()),
                        IdString::Named("a".to_string()),
                    );
                    map.insert(
                        IdString::Named("b".to_string()),
                        IdString::Named("b".to_string()),
                    );
                    map
                },
                cell_map: {
                    let mut map = HashMap::new();
                    map.insert(
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/and.v".to_string(),
                                line: "9".to_string(),
                                id: "41".to_string(),
                            },
                            cell_index: 553,
                        },
                        SanitizedCellData {
                            cell_name: IdString::Unnamed {
                                gate_name: "and".to_string(),
                                file_path: "svql_query/verilog/many_ands.v".to_string(),
                                line: "16".to_string(),
                                id: "1".to_string(),
                            },
                            cell_index: 545,
                        },
                    );
                    map
                },
            },
        ];

        Ok(DriverIterator::new(matches))
    }
}
