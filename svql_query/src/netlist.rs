use std::path::Path;

use svql_driver::{context::Context, prelude::Driver};
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

    fn query<'p, 'd>(
        haystack_path: &Path,
        haystack_module_name: &str,
        context: &Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'p, 'd>> {
        let pattern = context.get_ref(path)

        find_subgraphs(pattern_path, haystack_path, config)
            .iter()
            .map(|m| Self::from_subgraph(m, path.clone()))
            .collect()
    }

    fn context(&self, driver: &Driver) -> Context {
        todo!()
    }
}
