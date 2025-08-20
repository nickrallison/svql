use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use prjunnamed_netlist::Design;
use svql_driver::prelude::Driver;
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
}

pub trait SearchableNetlist: NetlistMeta + Sized {
    type Hit<'p, 'd>;

    fn from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;

    fn query<'ctx>(
        driver: Driver,
        haystack_module_name: &str,
        haystack_path: &Path,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>> {
        let haystack: Arc<Design> = driver
            .get_by_path(haystack_path, haystack_module_name)
            .expect("Failed to get haystack from driver");

        let needle: Arc<Design> = driver
            .get_by_path(&PathBuf::from(Self::FILE_PATH), Self::MODULE_NAME)
            .expect("Failed to get needle from driver");

        let subgraphs = find_subgraphs(needle.as_ref(), haystack.as_ref(), config)
            .iter()
            .map(|m| Self::from_subgraph(m, path.clone()))
            .collect::<Vec<_>>();

        subgraphs
    }
}

pub struct NetlistResultWrapper<T> {
    pub results: Vec<T>,
    pub _needle: Arc<Design>,
    pub _haystack: Arc<Design>,
}
