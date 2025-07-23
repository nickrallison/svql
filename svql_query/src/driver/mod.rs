use crate::driver::net::{NetDriver, SvqlDriverNetError};
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::SanitizedQueryMatch;

use thiserror::Error;

pub mod net;

pub enum Driver {
    Net(NetDriver),
}

impl Driver {
    pub fn new_net(addr: String) -> Self {
        Driver::Net(NetDriver::new(addr))
    }

    pub fn query(
        &self,
        cfg: &SvqlRuntimeConfig,
    ) -> Result<Vec<Result<SanitizedQueryMatch, DriverConversionError>>, DriverError> {
        match self {
            Driver::Net(driver) => driver.query(cfg).map_err(DriverError::NetError),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum DriverConversionError {
    #[error("Query match conversion error: {0}")]
    IdStringError(#[from] svql_common::mat::IdStringError),
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("{0}")]
    NetError(#[from] SvqlDriverNetError),
}
