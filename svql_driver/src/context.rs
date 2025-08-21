// svql_driver/src/context.rs
use std::collections::HashMap;
use std::sync::Arc;

use prjunnamed_netlist::Design;

use crate::driver::DriverKey;

#[derive(Debug, Clone)]
pub struct Context {
    designs: HashMap<DriverKey, Arc<Design>>,
}
