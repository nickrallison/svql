use crate::driver::{Driver, DriverConversionError, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;
use svql_common::config::ffi::SvqlRuntimeConfig;
use svql_common::mat::{IdString, SanitizedCellData, SanitizedQueryMatch};
use thiserror::Error;

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}

#[derive(Debug, Clone)]
pub struct RtlModule<ModuleType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub module: ModuleType,
}

impl<ModuleType> RtlModule<ModuleType>
where
    ModuleType: RtlModuleTrait<ModuleType>,
{
    pub fn new(inst: String, module: ModuleType) -> Self {
        RtlModule {
            inst,
            connections: EMPTY_CONNECTIONS.clone(),
            module,
        }
    }

    fn instance(&self, parent: Option<&str>) -> String {
        if let Some(p) = parent {
            format!("{}.{}", p, self.inst)
        } else {
            self.inst.clone()
        }
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }

    fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        cfg.pat_filename = self.module.file_path().to_string_lossy().into_owned();
        cfg.pat_module_name = self.module.module_name().to_string();
        cfg.verbose = true;
        cfg
    }

    pub fn query<ModuleResultType>(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlModuleResult<ModuleResultType>, QueryError>>, DriverError>
    where
        ModuleResultType: Debug,
        ModuleResultType: RtlModuleResultTrait<ModuleResultType>,
    {
        let cfg = self.config();
        let mut matches = driver.query(&cfg)?;
        let mut results = Vec::new();
        for m in matches {
            match m {
                Ok(m) => {
                    let match_ = RtlModuleResult::from_match(m);
                    if let Ok(result) = match_ {
                        results.push(Ok(result));
                    } else {
                        let err = match_.expect_err("Match Error");
                        results.push(Err(err));
                    }
                }
                Err(e) => results.push(Err(QueryError::DriverConversionError(e))),
            }
        }
        Ok(results)
    }
}

pub trait RtlModuleTrait<ModuleType> {
    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct RtlModuleResult<ModuleResultType> {
    // pub inst: String,
    pub cells: Vec<SanitizedCellData>,
    pub module: ModuleResultType,
}

impl<ModuleResultType> RtlModuleResult<ModuleResultType>
where
    ModuleResultType: RtlModuleResultTrait<ModuleResultType>,
{
    fn new(cells: Vec<SanitizedCellData>, module: ModuleResultType) -> Self {
        RtlModuleResult { cells, module }
    }
    fn from_match(m: SanitizedQueryMatch) -> Result<RtlModuleResult<ModuleResultType>, QueryError> {
        let cell_map = m.cell_map;
        let port_map = m.port_map;
        let module_result = ModuleResultType::from_portmap(port_map)?;
        let cells = cell_map.into_values().collect();
        Ok(RtlModuleResult {
            cells,
            module: module_result,
        })
    }
}

pub trait RtlModuleResultTrait<ModuleResultType> {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Result<ModuleResultType, QueryError>;
}

// ###################

#[derive(Debug, Clone)]
pub struct RtlQuery<QueryType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub query: QueryType,
}

impl<QueryType> RtlQuery<QueryType>
where
    QueryType: RtlQueryTrait<QueryType>,
{
    pub fn new(inst: String, query: QueryType) -> Self {
        RtlQuery {
            inst,
            connections: EMPTY_CONNECTIONS.clone(),
            query,
        }
    }

    fn instance(&self, parent: Option<&str>) -> String {
        if let Some(p) = parent {
            format!("{}.{}", p, self.inst)
        } else {
            self.inst.clone()
        }
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }

    pub fn query<QueryResultType>(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlQueryResult<QueryResultType>, QueryError>>, DriverError>
    where
        QueryResultType: Debug,
        QueryResultType: RtlQueryResultTrait<QueryResultType>,
    {
        self.query.run_query(driver)
    }
}

pub trait RtlQueryTrait<QueryType> {
    fn run_query<QueryResultType>(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlQueryResult<QueryResultType>, QueryError>>, DriverError>
    where
        QueryResultType: Debug,
        QueryResultType: RtlQueryResultTrait<QueryResultType>;
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
}

#[derive(Debug)]
pub struct RtlQueryResult<QueryResultType> {
    pub cells: Vec<SanitizedCellData>,
    pub query: QueryResultType,
}

impl<QueryResultType> RtlQueryResult<QueryResultType>
where
    QueryResultType: RtlQueryResultTrait<QueryResultType>,
{
    fn new(cells: Vec<SanitizedCellData>, query: QueryResultType) -> Self {
        RtlQueryResult { cells, query }
    }
}

pub trait RtlQueryResultTrait<QueryResultType> {}

#[derive(Debug, Error)]
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
