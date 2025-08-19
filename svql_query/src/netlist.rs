use svql_driver::prelude::Driver;
use svql_subgraph::{SubgraphMatch, config, find_subgraphs};

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
}

pub trait SearchableNetlist: NetlistMeta + Sized {
    type Hit<'p, 'd>;

    fn from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;

    /// Default query that preserves existing behavior (superset arity + Full dedupe).
    fn query<'p, 'd>(
        pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
        config: &config::Config,
    ) -> Vec<Self::Hit<'p, 'd>> {
        Self::query_with_config(pattern, haystack, path, &config)
    }

    /// New helper to allow callers to select arity and dedupe policy.
    fn query_with_config<'p, 'd>(
        pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
        config: &config::Config,
    ) -> Vec<Self::Hit<'p, 'd>> {
        find_subgraphs(pattern.design_as_ref(), haystack.design_as_ref(), config)
            .iter()
            .map(|m| Self::from_subgraph(m, path.clone()))
            .collect()
    }
}
