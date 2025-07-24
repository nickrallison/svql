pub mod result;
pub mod traits;

use crate::driver::{Driver, DriverConversionError, DriverError};
use crate::module::result::RtlModuleResult;
use crate::module::traits::RtlModuleTrait;
use crate::ports::{Connection, InPort, OutPort};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::IdString;
use thiserror::Error;

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}

#[derive(Debug, Clone)]
pub struct RtlModule<ModuleType> {
    pub inst: Arc<String>,
    pub full_path: Vec<Arc<String>>,
    // ################
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub module: ModuleType,
}

impl<ModuleType> RtlModule<ModuleType>
where
    ModuleType: RtlModuleTrait,
{
    pub fn new(module: ModuleType, inst: String) -> Self {
        let mut module = RtlModule {
            inst: Arc::new(inst),
            full_path: vec![],
            connections: EMPTY_CONNECTIONS.clone(),
            module,
        };
        module.init_full_path(vec![]);
        module
    }

    // pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
    //     self.connections.insert(conn);
    // }

    pub(crate) fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        cfg.pat_filename = self.module.file_path().to_string_lossy().into_owned();
        cfg.pat_module_name = self.module.module_name().to_string();
        cfg.verbose = true;
        cfg
    }

    pub(crate) fn init_full_path(&mut self, parent_path: Vec<Arc<String>>) {
        let mut full_path = parent_path.clone();
        full_path.push(self.inst.clone());
        self.full_path = full_path.clone();

        // Initialize full path for module's ports
        self.module.init_full_path(full_path);
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<RtlModuleResult<ModuleType::Result>>, DriverError> {
        let cfg = self.config();
        let matches = driver.query(&cfg)?;

        let inst = self.inst.clone();
        let full_path = self.full_path.clone();

        let iter = matches
            .into_iter()
            .map(RtlModuleResult::from_match)
            .map(|mut result| {
                result.inst = inst.clone();
                result.full_path = full_path.clone();
                result
            })
            .collect();
        Ok(iter)
    }
}

#[derive(Debug, Clone, Error)]
pub enum QueryError {
    #[error("Cannot convert Query Match PortMap: {0:#?}, due to missing port `{1}`")]
    MissingPort(HashMap<IdString, IdString>, String),
    #[error("Query match conversion error: {0}")]
    DriverConversionError(#[from] DriverConversionError),
}

pub fn lookup(m: &HashMap<IdString, IdString>, pin: &str) -> Result<IdString, QueryError> {
    m.get(&IdString::Named(pin.into()))
        .cloned()
        .ok_or_else(|| QueryError::MissingPort(m.clone(), pin.to_string()))
}

pub(crate) fn instance(inst: &str, parent: Option<&str>) -> String {
    if let Some(p) = parent {
        format!("{}.{}", p, inst)
    } else {
        inst.to_string()
    }
}
