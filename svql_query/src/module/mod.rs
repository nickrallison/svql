pub mod result;
pub mod traits;

use crate::driver::{Driver, DriverConversionError, DriverError};
use crate::module::result::RtlModuleResult;
use crate::module::traits::RtlModuleTrait;
use crate::ports::{Connection, InPort, OutPort};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::matches::IdString;
use thiserror::Error;

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}

#[derive(Debug, Clone)]
pub struct RtlModule<ModuleType> {
    pub height: usize,
    pub inst: Arc<String>,
    pub full_path: VecDeque<Arc<String>>,
    // ################
    // pub connections: HashSet<Connection<InPort, OutPort>>,
    pub module: ModuleType,
}

impl<ModuleType> RtlModule<ModuleType>
where
    ModuleType: RtlModuleTrait,
{
    pub fn new(module: ModuleType, inst: String) -> Self {
        let mut module = RtlModule {
            inst: Arc::new(inst),
            full_path: vec![].into(),
            // connections: EMPTY_CONNECTIONS.clone(),
            module,
            height: 0,
        };
        module.init_full_path(vec![].into(), 0);
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

    pub(crate) fn init_full_path(&mut self, parent_path: VecDeque<Arc<String>>, height: usize) {
        let mut full_path = parent_path.clone();
        full_path.push_back(self.inst.clone());
        self.full_path = full_path.clone();
        self.height = height;

        // Initialize full path for module's ports
        self.module.init_full_path(full_path, height);
    }

    #[allow(dead_code)]
    pub fn inst_path(&self) -> String {
        inst_path(&self.full_path)
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

pub fn inst_path(full_path: &VecDeque<Arc<String>>) -> String {
    full_path
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(".")
}
