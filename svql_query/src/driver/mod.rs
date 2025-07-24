use crate::driver::net::{NetDriver, SvqlDriverNetError};
use crate::driver::mock::MockDriver;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::SanitizedQueryMatch;

use thiserror::Error;

pub mod net;
pub mod mock;

pub enum Driver {
    Net(NetDriver),
    Mock(MockDriver),
}

impl Driver {
    pub fn new_net(addr: String) -> Self {
        Driver::Net(NetDriver::new(addr))
    }
    
    pub fn new_mock() -> Self {
        Driver::Mock(MockDriver::new())
    }

    pub fn query(&self, cfg: &SvqlRuntimeConfig) -> Result<Vec<SanitizedQueryMatch>, DriverError> {
        match self {
            Driver::Net(driver) => driver.query(cfg).map_err(DriverError::NetError),
            Driver::Mock(driver) => driver.query(cfg).map_err(DriverError::NetError),
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
