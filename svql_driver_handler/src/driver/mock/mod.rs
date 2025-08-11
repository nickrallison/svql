use crate::driver::mock::and_three::MockDriverThreeAnd;
use crate::driver::mock::or_three::MockDriverThreeOr;
use crate::driver::net::SvqlDriverNetError;
use crate::driver::DriverIterator;
use std::collections::HashMap;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::id_string::IdString;
use svql_common::matches::{SanitizedCellData, SanitizedQueryMatch};

pub mod and_three;
pub mod or_three;

pub enum MockDriver {
    ThreeAnd(MockDriverThreeAnd),
    ThreeOr(MockDriverThreeOr),
}

impl MockDriver {
    pub fn query(&self, cfg: &SvqlRuntimeConfig) -> Result<DriverIterator, SvqlDriverNetError> {
        match self {
            MockDriver::ThreeAnd(driver) => driver.query(cfg),
            MockDriver::ThreeOr(driver) => driver.query(cfg),
        }
    }
}
