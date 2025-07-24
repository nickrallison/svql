use crate::driver::mock::MockDriver;
use crate::driver::net::{NetDriver, SvqlDriverNetError};
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::SanitizedQueryMatch;

use thiserror::Error;

pub mod mock;
pub mod net;

#[derive(Debug, Clone)]
pub struct DriverIterator {
    matches: Vec<SanitizedQueryMatch>,
}

impl Iterator for DriverIterator {
    type Item = SanitizedQueryMatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.matches.is_empty() {
            None
        } else {
            Some(self.matches.remove(0))
        }
    }
}

impl DriverIterator {
    pub fn new(matches: Vec<SanitizedQueryMatch>) -> Self {
        DriverIterator { matches }
    }

    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<SanitizedQueryMatch> {
        self.matches.iter()
    }
}

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

    pub fn query(&self, cfg: &SvqlRuntimeConfig) -> Result<DriverIterator, DriverError> {
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
