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

pub trait RtlModuleTrait {
    type Result: Debug + RtlModuleResultTrait;

    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;
}

pub trait RtlModuleResultTrait: Sized {
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
    ) -> Result<Vec<RtlModuleResult<ModuleType::Result>>, DriverError> {
        let cfg = self.config();
        let matches = driver.query(&cfg)?;
        Ok(matches
            .into_iter()
            // RtlModuleResult<ModuleType::Result>
            .map(|m| RtlModuleResult::from_match(m))
            .collect())
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

// ###################

// Define a trait for queryable components (both modules and queries)
pub trait Queryable {
    fn query(&self, driver: &Driver) -> Result<Vec<SanitizedQueryMatch>, DriverError>;
}

impl<M: RtlModuleTrait> Queryable for RtlModule<M> {
    fn query(&self, driver: &Driver) -> Result<Vec<SanitizedQueryMatch>, DriverError> {
        let cfg = self.config();
        driver.query(&cfg)
    }
}

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;

    /// Get submodules that are part of this query
    fn sub_modules(&self) -> Vec<&dyn Queryable>;

    /// Get subqueries that are part of this query
    fn sub_queries(&self) -> Vec<&dyn Queryable>;
}

pub trait RtlQueryResultTrait {}

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

    pub fn query(
        &self,
        _driver: &Driver,
    ) -> Result<Vec<RtlQueryResult<QueryType::Result>>, DriverError> {
        todo!("Implement RtlQuery::query method");
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
