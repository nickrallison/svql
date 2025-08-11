
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

lazy_static! {
    static ref NAMED_IDSTRING_RE: Regex = Regex::new(r"^\\(\S*)$").unwrap();
    static ref UNNAMED_LOCATION_IDSTRING_RE: Regex =
        Regex::new(r"^\$([^\$]*)\$([^:]*):([^\$]*)\$(.*)$").unwrap();
    static ref UNNAMED_NO_LOCATION_IDSTRING_RE: Regex =
        Regex::new(r"^\$([^\$]*)\$(.*)$").unwrap();
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum IdString {
    // "\\[name]"
    Named(String),
    // $and$examples/patterns/basic/and/verilog/many_ands.v:14$2_Y
    // $and$examples/patterns/basic/and/verilog/and.v:9$11
    // $and$examples/patterns/basic/and/verilog/many_ands.v:14$2
    UnnamedLocation {
        gate_name: String,
        file_path: String,
        line: String,
        id: String,
    },
    // $procdff$22 - usually from opt as far as I can tell
    UnnamedNoLocation {
        gate_name: String,
        id: String,
    },
}

impl Display for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdString::Named(name) => write!(f, "\\{name}"),
            IdString::UnnamedLocation {
                gate_name,
                file_path,
                line,
                id,
            } => {
                write!(f, "${gate_name}${file_path}:{line}${id}")
            },
            IdString::UnnamedNoLocation { gate_name, id } => {
                write!(f, "${gate_name}${id}")
            },
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum IdStringError {
    #[error("{0}")]
    InvalidFormat(String),
}

impl TryFrom<&str> for IdString {
    type Error = IdStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(caps) = NAMED_IDSTRING_RE.captures(value) {
            return Ok(IdString::Named(caps[1].to_string()));
        } else if let Some(caps) = UNNAMED_LOCATION_IDSTRING_RE.captures(value) {
            return Ok(IdString::UnnamedLocation {
                gate_name: caps[1].to_string(),
                file_path: caps[2].to_string(),
                line: caps[3].to_string(),
                id: caps[4].to_string(),
            });
        } else if let Some(caps) = UNNAMED_NO_LOCATION_IDSTRING_RE.captures(value) {
            return Ok(IdString::UnnamedNoLocation {
                gate_name: caps[1].to_string(),
                id: caps[2].to_string(),
            });
        }
        Err(IdStringError::InvalidFormat(value.to_string()))
    }
}

impl From<String> for IdString {
    fn from(value: String) -> Self {
        IdString::try_from(value.as_str()).unwrap_or_else(|_| IdString::Named(value))
    }
}

pub fn parse_idstring(id_string: &str) -> Result<IdString, IdStringError> {
    IdString::try_from(id_string)
}
