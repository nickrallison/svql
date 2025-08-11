use crate::id_string::{parse_idstring, IdString, IdStringError};
use crate::matches::ffi::{CellData, QueryMatchList};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[cxx::bridge]
pub mod ffi {

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct QueryMatchList {
        pub matches: Vec<QueryMatch>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct QueryMatch {
        pub port_map: Vec<StringPair>,
        pub cell_map: Vec<CellPair>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct CellData {
        pub cell_name: String,
        pub cell_index: usize,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct StringPair {
        pub needle: String,
        pub haystack: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct CellPair {
        pub needle: CellData,
        pub haystack: CellData,
    }

    extern "Rust" {
        fn matchlist_into_json_string(cfg: &QueryMatchList) -> String;
        fn matchlist_from_json_string(json: &str) -> QueryMatchList;
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct SanitizedCellData {
    pub cell_name: IdString,
    pub cell_index: usize,
}

impl Display for SanitizedCellData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.cell_name, self.cell_index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizedQueryMatch {
    pub port_map: HashMap<IdString, IdString>,
    pub cell_map: HashMap<SanitizedCellData, SanitizedCellData>,
}

impl Display for SanitizedQueryMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let port_map: Vec<String> = self
            .port_map
            .iter()
            .map(|(k, v)| format!("{k} -> {v}"))
            .collect();
        let cell_map: Vec<String> = self
            .cell_map
            .iter()
            .map(|(k, v)| format!("{k} -> {v}"))
            .collect();
        write!(
            f,
            "Port Map: [{}], \nCell Map: [{}]",
            port_map.join(", "),
            cell_map.join(", ")
        )
    }
}

// into iter
impl TryInto<Vec<SanitizedQueryMatch>> for ffi::QueryMatchList {
    type Error = IdStringError;
    fn try_into(self) -> Result<Vec<SanitizedQueryMatch>, Self::Error> {
        let matches = self
            .matches
            .into_iter()
            .map(|m| m.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(matches)
    }
}

impl TryInto<SanitizedQueryMatch> for ffi::QueryMatch {
    type Error = IdStringError;
    fn try_into(self) -> Result<SanitizedQueryMatch, Self::Error> {
        let port_map = self
            .port_map
            .into_iter()
            .map(|pair| {
                let needle: IdString = parse_idstring(&pair.needle)?;
                let haystack: IdString = parse_idstring(&pair.haystack)?;
                Ok((needle, haystack))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let cell_map = self
            .cell_map
            .into_iter()
            .map(|pair| {
                let needle: SanitizedCellData = pair.needle.try_into()?;
                let haystack: SanitizedCellData = pair.haystack.try_into()?;
                Ok((needle, haystack))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(SanitizedQueryMatch { port_map, cell_map })
    }
}

impl TryInto<SanitizedCellData> for CellData {
    type Error = IdStringError;
    fn try_into(self) -> Result<SanitizedCellData, Self::Error> {
        Ok(SanitizedCellData {
            cell_name: parse_idstring(&self.cell_name)?,
            cell_index: self.cell_index,
        })
    }
}

pub fn matchlist_into_json_string(cfg: &QueryMatchList) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize QueryMatchList to JSON")
}

pub fn matchlist_from_json_string(json: &str) -> QueryMatchList {
    serde_json::from_str(json).expect("Failed to deserialize JSON to QueryMatchList")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matches::ffi::{CellData, CellPair, QueryMatch, QueryMatchList, StringPair};

    #[test]
    fn test_parse_named_idstring() {
        let named_id = "\\test_signal";
        let result = named_id.try_into().unwrap();
        match result {
            IdString::Named(name) => assert_eq!(name, "test_signal"),
            _ => panic!("Expected Named variant"),
        }
    }

    #[test]
    fn test_parse_unnamed_location_idstring() {
        let unnamed_id = "$and$examples/patterns/basic/and/verilog/many_ands.v:14$2_Y";
        let result = unnamed_id.try_into().unwrap();
        match result {
            IdString::UnnamedLocation {
                gate_name,
                file_path,
                line,
                id,
            } => {
                assert_eq!(gate_name, "and");
                assert_eq!(file_path, "examples/patterns/basic/and/verilog/many_ands.v");
                assert_eq!(line, "14");
                assert_eq!(id, "2_Y");
            }
            _ => panic!("Expected UnnamedLocation variant"),
        }
    }

    #[test]
    fn test_parse_unnamed_location_idstring_simple() {
        let unnamed_id = "$and$examples/patterns/basic/and/verilog/and.v:9$11";
        let result = unnamed_id.try_into().unwrap();
        match result {
            IdString::UnnamedLocation {
                gate_name,
                file_path,
                line,
                id,
            } => {
                assert_eq!(gate_name, "and");
                assert_eq!(file_path, "examples/patterns/basic/and/verilog/and.v");
                assert_eq!(line, "9");
                assert_eq!(id, "11");
            }
            _ => panic!("Expected UnnamedLocation variant"),
        }
    }

    #[test]
    fn test_parse_unnamed_no_location_idstring() {
        let unnamed_id = "$procdff$22";
        let result = unnamed_id.try_into().unwrap();
        match result {
            IdString::UnnamedNoLocation { gate_name, id } => {
                assert_eq!(gate_name, "procdff");
                assert_eq!(id, "22");
            }
            _ => panic!("Expected UnnamedNoLocation variant"),
        }
    }

    #[test]
    fn test_parse_invalid_idstring() {
        let invalid_id = "invalid_format";
        let result: Result<IdString, IdStringError> = invalid_id.try_into();
        assert!(result.is_err());
        match result.unwrap_err() {
            IdStringError::InvalidFormat(msg) => assert_eq!(msg, "invalid_format"),
        }
    }

    #[test]
    fn test_matchlist_json_serialization() {
        let match_list = QueryMatchList {
            matches: vec![QueryMatch {
                port_map: vec![StringPair {
                    needle: "\\input".to_string(),
                    haystack: "\\A".to_string(),
                }],
                cell_map: vec![CellPair {
                    needle: CellData {
                        cell_name: "\\and_gate".to_string(),
                        cell_index: 0,
                    },
                    haystack: CellData {
                        cell_name: "\\and_inst".to_string(),
                        cell_index: 1,
                    },
                }],
            }],
        };

        let json_string = matchlist_into_json_string(&match_list);
        assert!(json_string.contains("input"));
        assert!(json_string.contains("and_gate"));

        let deserialized = matchlist_from_json_string(&json_string);
        assert_eq!(deserialized.matches.len(), 1);
        assert_eq!(deserialized.matches[0].port_map.len(), 1);
        assert_eq!(deserialized.matches[0].cell_map.len(), 1);
        assert_eq!(deserialized.matches[0].port_map[0].needle, "\\input");
        assert_eq!(deserialized.matches[0].port_map[0].haystack, "\\A");
    }

    #[test]
    fn test_matchlist_roundtrip_serialization() {
        let original_list = QueryMatchList {
            matches: vec![QueryMatch {
                port_map: vec![
                    StringPair {
                        needle: "\\a".to_string(),
                        haystack: "\\input1".to_string(),
                    },
                    StringPair {
                        needle: "\\b".to_string(),
                        haystack: "\\input2".to_string(),
                    },
                ],
                cell_map: vec![CellPair {
                    needle: CellData {
                        cell_name: "\\gate1".to_string(),
                        cell_index: 0,
                    },
                    haystack: CellData {
                        cell_name: "\\gate2".to_string(),
                        cell_index: 5,
                    },
                }],
            }],
        };

        let json_string = matchlist_into_json_string(&original_list);
        let roundtrip_list = matchlist_from_json_string(&json_string);

        assert_eq!(original_list.matches.len(), roundtrip_list.matches.len());
        let original_match = &original_list.matches[0];
        let roundtrip_match = &roundtrip_list.matches[0];

        assert_eq!(
            original_match.port_map.len(),
            roundtrip_match.port_map.len()
        );
        assert_eq!(
            original_match.cell_map.len(),
            roundtrip_match.cell_map.len()
        );

        // Verify port map preservation
        for (orig, rt) in original_match
            .port_map
            .iter()
            .zip(roundtrip_match.port_map.iter())
        {
            assert_eq!(orig.needle, rt.needle);
            assert_eq!(orig.haystack, rt.haystack);
        }

        // Verify cell map preservation
        for (orig, rt) in original_match
            .cell_map
            .iter()
            .zip(roundtrip_match.cell_map.iter())
        {
            assert_eq!(orig.needle.cell_name, rt.needle.cell_name);
            assert_eq!(orig.needle.cell_index, rt.needle.cell_index);
            assert_eq!(orig.haystack.cell_name, rt.haystack.cell_name);
            assert_eq!(orig.haystack.cell_index, rt.haystack.cell_index);
        }
    }

    #[test]
    fn test_empty_query_match_list() {
        let empty_list = QueryMatchList { matches: vec![] };

        let json_string = matchlist_into_json_string(&empty_list);
        let deserialized = matchlist_from_json_string(&json_string);
        assert_eq!(deserialized.matches.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Failed to deserialize JSON to QueryMatchList")]
    fn test_matchlist_invalid_json() {
        matchlist_from_json_string("invalid json");
    }
}
