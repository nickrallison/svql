use std::path::Path;

use svql_driver::{Driver, context::Context};
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

    #[allow(unreachable_code)]
    fn query<'p, 'd>(
        _haystack_path: &Path,
        _haystack_module_name: &str,
        _context: &Context,
        _path: Instance,
        _config: &Config,
    ) -> Vec<Self::Hit<'p, 'd>> {
        let _needle = todo!("Get needle_path from context");
        let _haystack = todo!("Get haystack_path from context");

        find_subgraphs(_needle, _haystack, _config)
            .iter()
            .map(|m| Self::from_subgraph(m, _path.clone()))
            .collect()
    }

    fn context(_driver: &Driver) -> Context {
        todo!("Get context from driver, return single context item containing self")
    }
}
