#![allow(dead_code)]

use std::path::PathBuf;

use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::id_string::IdStringError;
use svql_common::matches::SanitizedQueryMatch;

use thiserror::Error;

use crate::net::{NetDriver, SvqlDriverNetError};
use crate::proc::ProcDriver;

pub mod mock;
pub mod net;
pub mod proc;
pub mod prjunnamed;

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
}

#[derive(Debug)]
pub enum Driver {
    Net(NetDriver),
    Mock(mock::MockDriver),
    Proc(ProcDriver),
    // PrjUnnamed(PrjUnnamedDriver),
}

impl Driver {
    pub fn new_net(addr: String) -> Self {
        Driver::Net(NetDriver::new(addr))
    }

    pub fn new_mock(mock: mock::MockDriver) -> Self {
        Driver::Mock(mock)
    }

    pub fn new_proc(design: PathBuf, module_name: String) -> Result<Self, String> {
        let proc_driver = ProcDriver::new(design, module_name)?;
        Ok(Driver::Proc(proc_driver))
    }

    pub fn query(&self, cfg: &SvqlRuntimeConfig) -> Result<DriverIterator, DriverError> {
        match self {
            Driver::Net(driver) => driver.query(cfg).map_err(DriverError::NetError),
            Driver::Mock(driver) => driver.query(cfg).map_err(DriverError::NetError),
            Driver::Proc(driver) => driver.query(cfg).map_err(DriverError::NetError),
            // Driver::PrjUnnamed(driver) => driver.query(cfg).map_err(DriverError::NetError),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum DriverConversionError {
    #[error("Query match conversion error: {0}")]
    IdStringError(#[from] IdStringError),
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("{0}")]
    NetError(#[from] SvqlDriverNetError),
}
