use svql_driver::{Driver, DriverKey, context::Context};
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

    fn driver_key() -> DriverKey {
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }
}

pub trait SearchableNetlist: NetlistMeta + Sized {
    type Hit<'p, 'd>;

    fn from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>> {
        let needle = context
            .get(&Self::driver_key())
            .expect("Pattern design not found in context")
            .as_ref();
        let haystack = context
            .get(haystack_key)
            .expect("Haystack design not found in context")
            .as_ref();

        find_subgraphs(needle, haystack, config)
            .into_iter()
            .map(|m| Self::from_subgraph(&m, path.clone()))
            .collect()
    }

    fn context(driver: &Driver) -> Context {
        let key = Self::driver_key();
        if let Some(design) = driver.get_design(&key) {
            Context::from_single(key, design)
        } else {
            todo!(
                "Design is not loaded, at a later point Implement some live design loading so this point is not reached"
            );
        }
    }
}
