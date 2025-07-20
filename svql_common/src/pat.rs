// pattern.rs

use serde::{Serialize, Deserialize};
use std::path::{PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub file_loc: PathBuf,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub inout_ports: Vec<String>,
}
