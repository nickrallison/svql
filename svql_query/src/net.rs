use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};

// use anyhow::{anyhow, Context, Result};

use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::ffi::QueryMatchList;
use svql_common::mat::matchlist_from_json_string;
use svql_common::config::svql_runtime_config_into_json_string;

#[derive(Debug)]
pub enum SvqlQueryError {
    ConnectionError(String),
    ResponseError(String),
    ParseError(String),
}


pub fn run_svql_query<A: ToSocketAddrs>(
    addr: A,
    cfg: &SvqlRuntimeConfig,
) -> Result<QueryMatchList, SvqlQueryError> {
    // 1. serialise the request
    let json_cfg = svql_runtime_config_into_json_string(cfg);
    let mut stream = TcpStream::connect(addr)
        .map_err(|e| SvqlQueryError::ConnectionError(e.to_string()))?;

    // 2. send it (driver expects '\n' terminated line)
    stream
        .write_all(json_cfg.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|e| SvqlQueryError::ConnectionError(e.to_string()))?;

    // 3. read the single-line response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| SvqlQueryError::ConnectionError(e.to_string()))?;

    // 4. interpret
    if let Some(rest) = response.strip_prefix("ERR:") {
        return Err(SvqlQueryError::ResponseError(
            rest.trim().to_string(),
        ));
    } else if response.is_empty() {
        return Err(SvqlQueryError::ResponseError(
            "empty response".to_string(),
        ));
    }

    // 5. parse JSON to QueryMatchList
    let match_list =
        matchlist_from_json_string(response.trim());
    Ok(match_list)
}