use svql_driver::prelude::{DesignKey, Driver};
use svql_driver::query_ctx::QueryCtx;
use svql_subgraph::{Config, SubgraphMatch, find_subgraphs};

use crate::instance::Instance;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDir {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortSpec {
    pub name: &'static str,
    pub dir: PortDir,
}

pub trait NetlistMeta {
    const MODULE_NAME: &'static str;
    const FILE_PATH: &'static str;
    const PORTS: &'static [PortSpec];

    /// Load this netlist’s pattern into the shared Driver and return its key.
    /// Default impl uses FILE_PATH and MODULE_NAME.
    fn ensure_pattern_key(driver: &Driver) -> Result<DesignKey, Box<dyn std::error::Error>> {
        driver.ensure_loaded_with_top(
            std::path::PathBuf::from(Self::FILE_PATH),
            Self::MODULE_NAME.to_string(),
        )
    }
}

pub trait SearchableNetlist: NetlistMeta + Sized {
    type Hit<'p, 'd>;

    fn from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;

    fn query(driver: Driver, path: Instance, config: &Config) -> Vec<Self::Hit<'_, '_>> {

        
        find_subgraphs(driver.pat(), driver.hay(), config)
            .iter()
            .map(|m| Self::from_subgraph(m, path.clone()))
            .collect()
    }
}

/// Legacy helpers retained (thin wrappers around NetlistMeta defaults).
pub fn ensure_pattern_key<M: NetlistMeta>(
    driver: &Driver,
) -> Result<svql_driver::prelude::DesignKey, Box<dyn std::error::Error>> {
    M::ensure_pattern_key(driver)
}

pub fn ctx_for<M: NetlistMeta>(
    driver: &Driver,
    hay_key: &svql_driver::prelude::DesignKey,
) -> Result<QueryCtx, Box<dyn std::error::Error>> {
    M::open_ctx(driver, hay_key)
}
