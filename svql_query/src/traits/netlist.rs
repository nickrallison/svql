use svql_common::{Config, ModuleConfig};
use svql_driver::{Driver, DriverKey, context::Context, design_container::DesignContainer};
use svql_subgraph::{Embedding, EmbeddingSet};

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
        tracing::event!(tracing::Level::DEBUG, "Creating driver key for netlist");
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }
}

pub trait SearchableNetlist: NetlistMeta + Sized {
    type Hit<'ctx>;

    fn from_subgraph<'ctx>(
        m: &Embedding<'ctx, 'ctx>,
        path: Instance,
        embedding_set: &EmbeddingSet<'ctx, 'ctx>,
    ) -> Self::Hit<'ctx>;

    #[contracts::debug_requires(context.get(&Self::driver_key()).is_some(), "Pattern design must be present in context")]
    #[contracts::debug_requires(context.get(haystack_key).is_some(), "Haystack design must be present in context")]
    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info_span!("query", haystack = %haystack_key.module_name()).in_scope(|| {
            let needle_container: &DesignContainer = context
                .get(&Self::driver_key())
                .expect("Pattern design not found in context")
                .as_ref();
            let haystack_container: &DesignContainer = context
                .get(haystack_key)
                .expect("Haystack design not found in context")
                .as_ref();

            let needle = needle_container.design();
            let haystack = haystack_container.design();

            let needle_index = needle_container.index();
            let haystack_index = haystack_container.index();

            let embeddings = svql_subgraph::SubgraphMatcher::enumerate_with_indices(
                needle,
                haystack,
                needle_index,
                haystack_index,
                config,
            );
            tracing::debug!(
                num_raw_embeddings = embeddings.items.len(),
                needle_cells = needle_index.num_cells(),
                haystack_cells = haystack_index.num_cells(),
                "Raw subgraph matches before binding"
            );

            let hits: Vec<_> = embeddings
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, m)| {
                    tracing::trace!(embedding_idx = i, "Processing embedding");

                    let hit = Self::from_subgraph(m, path.clone(), &embeddings);
                    Some(hit)
                })
                .collect();

            tracing::info!(num_hits = hits.len(), "query complete");

            hits
        })
    }

    #[contracts::debug_ensures(ret.as_ref().map(|c| c.len()).unwrap_or(1) == 1, "Context for a single pattern only")]
    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        tracing::event!(tracing::Level::TRACE, "Creating context for netlist");
        let key = Self::driver_key();
        let design = driver
            .get_or_load_design(&key.path().display().to_string(), key.module_name(), config)?
            .1;

        Ok(Context::from_single(key, design))
    }
}
