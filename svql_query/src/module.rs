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
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Result<Self, QueryError>;
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

    fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        cfg.pat_filename = self.module.file_path().to_string_lossy().into_owned();
        cfg.pat_module_name = self.module.module_name().to_string();
        cfg.verbose = true;
        cfg
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlModuleResult<ModuleType::Result>, QueryError>>, DriverError> {
        let cfg = self.config();
        let matches = driver.query(&cfg)?;
        let mut out = Vec::new();

        for m in matches {
            match m {
                Ok(m) => match RtlModuleResult::<ModuleType::Result>::from_match(m) {
                    Ok(r) => out.push(Ok(r)),
                    Err(e) => out.push(Err(e)),
                },
                Err(e) => out.push(Err(QueryError::DriverConversionError(e))),
            }
        }

        Ok(out)
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
    fn from_match(m: SanitizedQueryMatch) -> Result<Self, QueryError> {
        let module = ModuleResultType::from_portmap(m.port_map)?;
        Ok(Self {
            cells: m.cell_map.into_values().collect(),
            module,
        })
    }
}

// ###################

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// Execute the query with the driver.  Implementors usually call
    /// `driver.query(..)` internally and translate the matches.
    fn run_query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlQueryResult<Self::Result>, QueryError>>, DriverError>;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
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
            connections: EMPTY_CONNECTIONS.clone(),
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
        driver: &Driver,
    ) -> Result<Vec<Result<RtlQueryResult<QueryType::Result>, QueryError>>, DriverError> {
        // Simply delegate to the concrete query implementation.
        self.query.run_query(driver)
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
