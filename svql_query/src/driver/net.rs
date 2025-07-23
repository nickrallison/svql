use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};

use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::config::svql_runtime_config_into_json_string;
use svql_common::mat::ffi::QueryMatchList;
use svql_common::mat::matchlist_from_json_string;
use svql_common::mat::SanitizedQueryMatch;

use thiserror::Error;

pub struct NetDriver {
    addr: String,
}

impl NetDriver {
    pub fn new(addr: String) -> Self {
        NetDriver { addr }
    }

    pub fn query(
        &self,
        cfg: &SvqlRuntimeConfig,
    ) -> Result<Vec<SanitizedQueryMatch>, SvqlDriverNetError> {
        run_svql_query_net(&self.addr, cfg)
    }
}

#[derive(Debug, Error)]
pub enum SvqlDriverNetError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Response error: {0}")]
    ResponseError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Query match conversion error: {0}")]
    IdStringError(#[from] svql_common::mat::IdStringError),
}

pub fn run_svql_query_net<A: ToSocketAddrs>(
    addr: A,
    cfg: &SvqlRuntimeConfig,
) -> Result<Vec<SanitizedQueryMatch>, SvqlDriverNetError> {
    // 1. serialise the request
    let json_cfg = svql_runtime_config_into_json_string(cfg);
    let mut stream =
        TcpStream::connect(addr).map_err(|e| SvqlDriverNetError::ConnectionError(e.to_string()))?;

    // 2. send it (driver expects '\n' terminated line)
    stream
        .write_all(format!("{}\n", json_cfg).as_bytes())
        .map_err(|e| SvqlDriverNetError::ConnectionError(e.to_string()))?;

    // 3. read the single-line response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| SvqlDriverNetError::ConnectionError(e.to_string()))?;

    // 4. interpret
    if response.starts_with("ERR:") {
        return Err(SvqlDriverNetError::ResponseError(
            response
                .strip_prefix("ERR:")
                .unwrap_or("")
                .trim()
                .to_string(),
        ));
    } else if response.is_empty() {
        return Err(SvqlDriverNetError::ResponseError(
            "empty response".to_string(),
        ));
    }

    // 5. parse JSON to QueryMatchList
    let match_list: QueryMatchList = matchlist_from_json_string(response.trim());

    let matches: Vec<SanitizedQueryMatch> = match_list
        .matches
        .into_iter()
        .map(|m| {
            m.try_into()
                .expect("QueryMatch::try_into SanitizedQueryMatch Failed")
        })
        .collect();
    Ok(matches)
}
