use crate::ports::{Connection, InPort, OutPort};
use crate::query::run_svql_query_leaf;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

use crate::driver::{Driver, DriverConversionError, DriverError};
use svql_common::mat::{IdString, SanitizedQueryMatch};

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}

pub trait RtlModule {
    type Result;
    fn instance(&self, parent: Option<&str>) -> String;
    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;

    fn from_match(m: SanitizedQueryMatch, inst: String) -> Result<Self::Result, QueryError>;

    fn query(&self, driver: &Driver) -> Result<Vec<Result<Self::Result, QueryError>>, DriverError> {
        let result = run_svql_query_leaf(driver, self.file_path(), self.module_name().to_string())?;
        let parent = None;
        let result_vec: Vec<Result<<Self as RtlModule>::Result, QueryError>> = result
            .into_iter()
            .map(|m| {
                if m.is_ok() {
                    Self::from_match(m.unwrap(), self.instance(parent).to_string())
                } else {
                    let err: DriverConversionError = m.expect_err("Expected an error");
                    let err = QueryError::DriverConversionError(err);
                    Err(err)
                }
            })
            .collect();
        Ok(result_vec)
    }
}
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Cannot convert Query Match: {0}, due to missing port `{1}`")]
    MissingPort(SanitizedQueryMatch, String),
    #[error("Query match conversion error: {0}")]
    DriverConversionError(#[from] DriverConversionError),
}

pub fn lookup(m: &SanitizedQueryMatch, pin: &str) -> Result<IdString, QueryError> {
    m.port_map
        .get(&IdString::Named(pin.into()))
        .cloned()
        .ok_or_else(|| QueryError::MissingPort(m.clone(), pin.to_string()))
}
