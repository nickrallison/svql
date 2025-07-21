use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mat::ffi::{QueryMatchList, CellData};
use thiserror::Error;
use std::fmt::Display;

lazy_static! {
    static ref NAMED_IDSTRING_RE:   Regex = Regex::new(r"^\\(\S*)$").unwrap();
    static ref UNNAMED_IDSTRING_RE: Regex = Regex::new(r"^\$([^\$]*)\$([^:]*):([^\$]*)\$(.*)$").unwrap();
}

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
pub enum IdString {
    // "\\[name]"
    Named(String),
    // $and$svql_query/verilog/many_ands.v:14$2_Y
    // $and$svql_query/verilog/and.v:9$11
    // $and$svql_query/verilog/many_ands.v:14$2
    Unnamed {
        gate_name: String,
        file_path: String,
        line: String,
        id: String,
    }
}

impl Display for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdString::Named(name) => write!(f, "\\{}", name),
            IdString::Unnamed { gate_name, file_path, line, id } => {
                write!(f, "${}${}:{}${}", gate_name, file_path, line, id)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum IdStringError {
    #[error("{0}")]
    InvalidFormat(String),
}

pub fn parse_idstring(idstring: &str) -> Result<IdString, IdStringError> {
    if let Some(caps) = NAMED_IDSTRING_RE.captures(idstring) {
        return Ok(IdString::Named(caps[1].to_string()));
    } else if let Some(caps) = UNNAMED_IDSTRING_RE.captures(idstring) {
        return Ok(IdString::Unnamed {
            gate_name: caps[1].to_string(),
            file_path: caps[2].to_string(),
            line: caps[3].to_string(),
            id: caps[4].to_string(),
        });
    }
    // panic!("Invalid idstring format: {}", idstring);
    Err(IdStringError::InvalidFormat(idstring.to_string()))
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
        let port_map: Vec<String> = self.port_map.iter()
            .map(|(k, v)| format!("{} -> {}", k, v))
            .collect();
        let cell_map: Vec<String> = self.cell_map.iter()
            .map(|(k, v)| format!("{} -> {}", k, v))
            .collect();
        write!(f, "Port Map: [{}], \nCell Map: [{}]",
               port_map.join(", "), cell_map.join(", "))
    }
}

// into iter
impl TryInto<Vec<SanitizedQueryMatch>> for ffi::QueryMatchList {
    type Error = IdStringError;
    fn try_into(self) -> Result<Vec<SanitizedQueryMatch>, Self::Error> {
        let matches = self.matches.into_iter().map(|m| m.try_into()).collect::<Result<Vec<_>, _>>()?;
        Ok(matches)
    }
}

impl TryInto<SanitizedQueryMatch> for ffi::QueryMatch {
    type Error = IdStringError;
    fn try_into(self) -> Result<SanitizedQueryMatch, Self::Error> {
        let port_map = self.port_map.into_iter()
            .map(|pair| {
                let needle: IdString = pair.needle.try_into()?;
                let haystack: IdString = pair.haystack.try_into()?;
                Ok((needle, haystack))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let cell_map = self.cell_map.into_iter()
            .map(|pair| {
                let needle: SanitizedCellData = pair.needle.try_into()?;
                let haystack: SanitizedCellData = pair.haystack.try_into()?;
                Ok((needle, haystack))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(SanitizedQueryMatch {
            port_map,
            cell_map,
        })
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

impl TryInto<IdString> for String {
    type Error = IdStringError;
    fn try_into(self) -> Result<IdString, Self::Error> {
        parse_idstring(&self)
    }
}

pub fn matchlist_into_json_string(cfg: &QueryMatchList) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize QueryMatchList to JSON")
}

pub fn matchlist_from_json_string(json: &str) -> QueryMatchList {
    serde_json::from_str(json).expect("Failed to deserialize JSON to QueryMatchList")
}