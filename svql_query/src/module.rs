use crate::driver::{Driver, DriverConversionError, DriverError, DriverIterator};
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
pub struct RtlModuleQueryIterator<T> {
    matches: std::iter::Map<DriverIterator, fn(SanitizedQueryMatch) -> RtlModuleResult<T>>,
}

impl<T> Iterator for RtlModuleQueryIterator<T> {
    type Item = RtlModuleResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.matches.next()
    }
}

pub struct RtlQueryQueryIterator<T> {
    matches: std::iter::Map<DriverIterator, fn(SanitizedQueryMatch) -> RtlQueryResult<T>>,
}

impl<T> Iterator for RtlQueryQueryIterator<T> {
    type Item = RtlQueryResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.matches.next()
    }
}

pub trait RtlModuleTrait {
    type Result: Debug + RtlModuleResultTrait;

    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;
}

pub trait RtlModuleResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
}

#[derive(Debug, Clone)]
pub struct RtlModule<ModuleType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub module: ModuleType,
}

impl<ModuleType> RtlModule<ModuleType>
where
    ModuleType: RtlModuleTrait,
{
    pub fn new(inst: String, module: ModuleType) -> Self {
        RtlModule {
            inst,
            connections: EMPTY_CONNECTIONS.clone(),
            module,
        }
    }

    fn instance(&self, parent: Option<&str>) -> String {
        instance(&self.inst, parent)
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }

    pub(crate) fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        cfg.pat_filename = self.module.file_path().to_string_lossy().into_owned();
        cfg.pat_module_name = self.module.module_name().to_string();
        cfg.verbose = true;
        cfg
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<RtlModuleQueryIterator<ModuleType::Result>, DriverError> {
        let cfg = self.config();
        let matches = driver.query(&cfg)?;
        let iter = matches.into_iter().map(
            RtlModuleResult::from_match
                as fn(SanitizedQueryMatch) -> RtlModuleResult<ModuleType::Result>,
        );
        Ok(RtlModuleQueryIterator { matches: iter })
    }
}

#[derive(Debug)]
pub struct RtlModuleResult<ModuleResultType> {
    // pub inst: String,
    pub cells: Vec<SanitizedCellData>,
    pub module: ModuleResultType,
}

impl<ModuleResultType> RtlModuleResult<ModuleResultType>
where
    ModuleResultType: RtlModuleResultTrait,
{
    fn new(cells: Vec<SanitizedCellData>, module: ModuleResultType) -> Self {
        RtlModuleResult { cells, module }
    }
    fn from_match(m: SanitizedQueryMatch) -> Self {
        let module = ModuleResultType::from_portmap(m.port_map);
        Self {
            cells: m.cell_map.into_values().collect(),
            module,
        }
    }
}

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn query(&self, driver: &Driver) -> Result<RtlQueryQueryIterator<Self::Result>, DriverError>;
}

pub trait RtlQueryResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
}

#[derive(Debug, Clone)]
pub struct RtlQuery<QueryType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub query: QueryType,
}

impl<QueryType> RtlQuery<QueryType>
where
    QueryType: RtlQueryTrait,
{
    pub fn new(inst: String, query: QueryType) -> Self {
        RtlQuery {
            inst,
            connections: QueryType::connect(&query),
            query,
        }
    }

    fn instance(&self, parent: Option<&str>) -> String {
        instance(&self.inst, parent)
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }

    pub(crate) fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        // For queries, we'll need to define how to get the file path and module name
        // This might need to be added to RtlQueryTrait or handled differently
        cfg.verbose = true;
        cfg
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<RtlQueryQueryIterator<QueryType::Result>, DriverError> {
        let cfg = self.config();
        let matches = driver.query(&cfg)?;
        Ok(RtlQueryQueryIterator {
            matches: matches.map(RtlQueryResult::from_match),
        })
    }
}

#[derive(Debug)]
pub struct RtlQueryResult<QueryResultType> {
    pub cells: Vec<SanitizedCellData>,
    pub query: QueryResultType,
}

impl<QueryResultType> RtlQueryResult<QueryResultType>
where
    QueryResultType: RtlQueryResultTrait,
{
    fn new(cells: Vec<SanitizedCellData>, query: QueryResultType) -> Self {
        RtlQueryResult { cells, query }
    }

    fn from_match(m: SanitizedQueryMatch) -> RtlQueryResult<QueryResultType> {
        let query = QueryResultType::from_portmap(m.port_map);
        Self {
            cells: m.cell_map.into_values().collect(),
            query,
        }
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

fn instance(inst: &str, parent: Option<&str>) -> String {
    if let Some(p) = parent {
        format!("{}.{}", p, inst)
    } else {
        inst.to_string()
    }
}

pub fn split_driver_results<Ok, Err>(res: Vec<Result<Ok, Err>>) -> (Vec<Ok>, Vec<Err>) {
    let mut oks = Vec::new();
    let mut errs = Vec::new();

    for r in res {
        match r {
            Ok(m) => oks.push(m),
            Err(e) => errs.push(e),
        }
    }
    (oks, errs)
}
