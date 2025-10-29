use svql_common::{Config, ModuleConfig};
use svql_driver::{context::Context, design_container::DesignContainer, Driver, DriverKey};
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

                    // Log matched cell type/port bindings
                    // if let Some(cell) = m.assignment.get_haystack_cell(m.needle_cells[0].clone()) {
                    //     // Assume first cell is the AND
                    //     tracing::trace!(
                    //         haystack_cell_type = format!("{:?}", cell.get().cell_type()),
                    //         haystack_cell_ports = format!("{:?}", cell.get().ports()),
                    //         "Matched AND cell details"
                    //     );
                    // }

                    let hit = Self::from_subgraph(m, path.clone(), &embeddings);
                    Some(hit)
                })
                .collect();

            tracing::info!(num_hits = hits.len(), "AndGate query complete");

            hits
        })
    }

    // #[contracts::debug_requires(context.get(&Self::driver_key()).is_some(), "Pattern design must be present in context")]
    // #[contracts::debug_requires(context.get(haystack_key).is_some(), "Haystack design must be present in context")]
    // fn query<'ctx>(
    //     haystack_key: &DriverKey,
    //     context: &'ctx Context,
    //     path: Instance,
    //     config: &Config,
    // ) -> Vec<Self::Hit<'ctx>> {
    //     tracing::event!(
    //         tracing::Level::TRACE,
    //         "Querying netlist with haystack key: {:?}",
    //         haystack_key
    //     );
    //     let needle_container: &DesignContainer = context
    //         .get(&Self::driver_key())
    //         .expect("Pattern design not found in context")
    //         .as_ref();
    //     let haystack_container: &DesignContainer = context
    //         .get(haystack_key)
    //         .expect("Haystack design not found in context")
    //         .as_ref();
    //
    //     let needle = needle_container.design();
    //     let haystack = haystack_container.design();
    //
    //     let needle_index = needle_container.index();
    //     let haystack_index = haystack_container.index();
    //
    //     let embeddings = svql_subgraph::SubgraphMatcher::enumerate_with_indices(
    //         needle,
    //         haystack,
    //         needle_index,
    //         haystack_index,
    //         config,
    //     );
    //
    //     embeddings
    //         .items
    //         .iter()
    //         .map(|m| Self::from_subgraph(m, path.clone(), &embeddings))
    //         .collect()
    // }

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

// #[macro_export]
// macro_rules! netlist {
//     (
//         name: $name:ident,
//         module_name: $module:expr,
//         file: $file:expr,
//         inputs: [$($input:ident),*],
//         outputs: [$($output:ident),*]
//     ) => {
//         #[derive(Debug, Clone)]
//         pub struct $name<S>
//         where
//             S: $crate::State,
//         {
//             pub path: $crate::instance::Instance,
//             $(pub $input: $crate::Wire<S>,)*
//             $(pub $output: $crate::Wire<S>,)*
//         }

//         impl<S> $name<S>
//         where
//             S: $crate::State,
//         {
//             pub fn new(path: $crate::instance::Instance) -> Self {
//                 Self {
//                     path: path.clone(),
//                     $($input: $crate::Wire::new(path.child(stringify!($input).to_string())),)*
//                     $($output: $crate::Wire::new(path.child(stringify!($output).to_string())),)*
//                 }
//             }
//         }

//         impl<S> $crate::WithPath<S> for $name<S>
//         where
//             S: $crate::State,
//         {
//             fn find_port(&self, p: &$crate::instance::Instance) -> Option<&$crate::Wire<S>> {
//                 let idx  = self.path.height() + 1;
//                 match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
//                     $(Some(stringify!($input)) => self.$input.find_port(p),)+
//                     $(Some(stringify!($output)) => self.$output.find_port(p),)+
//                     _ => None,
//                 }
//             }

//             fn path(&self) -> $crate::instance::Instance {
//                 self.path.clone()
//             }
//         }

//         impl $crate::netlist::NetlistMeta for $name<$crate::Search> {
//             const MODULE_NAME: &'static str = $module;
//             const FILE_PATH: &'static str = $file;

//             const PORTS: &'static [$crate::netlist::PortSpec] = &[
//                 $($crate::netlist::PortSpec {
//                     name: stringify!($input),
//                     dir: $crate::netlist::PortDir::In,
//                 },)*
//                 $($crate::netlist::PortSpec {
//                     name: stringify!($output),
//                     dir: $crate::netlist::PortDir::Out,
//                 },)*
//             ];
//         }

//         impl $crate::netlist::SearchableNetlist for $name<$crate::Search> {
//             type Hit<'ctx> = $name<$crate::Match<'ctx>>;

//             fn from_subgraph<'ctx>(
//                 m: &svql_subgraph::Embedding<'ctx, 'ctx>,
//                 path: $crate::instance::Instance,
//                 embedding_set: &svql_subgraph::EmbeddingSet<'ctx, 'ctx>,
//             ) -> Self::Hit<'ctx> {
//                 $(
//                     let $input = $crate::binding::bind_input(
//                         m,
//                         stringify!($input),
//                         0,
//                         &embedding_set.needle_input_fanout_by_name
//                     );
//                     let $input = $crate::Wire::with_val(
//                         path.child(stringify!($input).to_string()),
//                         $input
//                     );
//                 )*
//                 $(
//                     let $output = $crate::binding::bind_output(
//                         m,
//                         stringify!($output),
//                         0,
//                         &embedding_set.needle_output_fanin_by_name
//                     );
//                     let $output = $crate::Wire::with_val(
//                         path.child(stringify!($output).to_string()),
//                         $output
//                     );
//                 )*

//                 Self::Hit::<'ctx> {
//                     path: path.clone(),
//                     $($input,)*
//                     $($output,)*
//                 }
//             }
//         }
//     };
// }
