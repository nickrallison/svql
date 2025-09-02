use svql_common::Config;
use svql_driver::{context::Context, Driver, DriverKey};
use svql_subgraph::{find_subgraph_isomorphisms, SubgraphIsomorphism};

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

    fn from_subgraph<'ctx>(m: &SubgraphIsomorphism<'ctx, 'ctx>, path: Instance) -> Self::Hit<'ctx>;

    #[contracts::debug_requires(context.get(&Self::driver_key()).is_some(), "Pattern design must be present in context")]
    #[contracts::debug_requires(context.get(haystack_key).is_some(), "Haystack design must be present in context")]
    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::event!(
            tracing::Level::TRACE,
            "Querying netlist with haystack key: {:?}",
            haystack_key
        );
        let needle = context
            .get(&Self::driver_key())
            .expect("Pattern design not found in context")
            .as_ref();
        let haystack = context
            .get(haystack_key)
            .expect("Haystack design not found in context")
            .as_ref();

        find_subgraph_isomorphisms(needle, haystack, config)
            .into_iter()
            .map(|m| Self::from_subgraph(&m, path.clone()))
            .collect()
    }

    /// Same as `query`, but also updates the provided `progress` as the subgraph search proceeds.
    // #[contracts::debug_requires(context.get(&Self::driver_key()).is_some(), "Pattern design must be present in context")]
    // #[contracts::debug_requires(context.get(haystack_key).is_some(), "Haystack design must be present in context")]
    // fn query_with_progress<'ctx>(
    //     haystack_key: &DriverKey,
    //     context: &'ctx Context,
    //     path: Instance,
    //     config: &Config,
    // ) -> Vec<Self::Hit<'ctx>> {
    //     let needle = context
    //         .get(&Self::driver_key())
    //         .expect("Pattern design not found in context")
    //         .as_ref();
    //     let haystack = context
    //         .get(haystack_key)
    //         .expect("Haystack design not found in context")
    //         .as_ref();
    //
    //     svql_subgraph::find_subgraph_isomorphisms(needle, haystack, config, Some(progress))
    //         .into_iter()
    //         .map(|m| Self::from_subgraph(&m, path.clone()))
    //         .collect()
    // }

    #[contracts::debug_ensures(ret.as_ref().map(|c| c.len()).unwrap_or(1) == 1, "Context for a single pattern only")]
    fn context(driver: &Driver, config: &Config) -> Result<Context, Box<dyn std::error::Error>> {
        // tracing::event!(tracing::Level::TRACE, "Creating context for netlist");
        // let key = Self::driver_key();
        // let design = driver
        //     .get_or_load_design(key.path(), key.module_name().to_string(), config)?
        //     .1;
        //
        // Ok(Context::from_single(key, design))
        todo!()
    }
}

#[macro_export]
macro_rules! netlist {
    (
        name: $name:ident,
        module_name: $module:expr,
        file: $file:expr,
        inputs: [$($input:ident),*],
        outputs: [$($output:ident),*]
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name<S>
        where
            S: $crate::State,
        {
            pub path: $crate::instance::Instance,
            $(pub $input: $crate::Wire<S>,)*
            $(pub $output: $crate::Wire<S>,)*
        }

        impl<S> $name<S>
        where
            S: $crate::State,
        {
            pub fn new(path: $crate::instance::Instance) -> Self {
                Self {
                    path: path.clone(),
                    $($input: $crate::Wire::new(path.child(stringify!($input).to_string())),)*
                    $($output: $crate::Wire::new(path.child(stringify!($output).to_string())),)*
                }
            }
        }

        impl<S> $crate::WithPath<S> for $name<S>
        where
            S: $crate::State,
        {
            fn find_port(&self, p: &$crate::instance::Instance) -> Option<&$crate::Wire<S>> {
                let idx  = self.path.height() + 1;
                match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
                    $(Some(stringify!($input)) => self.$input.find_port(p),)+
                    $(Some(stringify!($output)) => self.$output.find_port(p),)+
                    _ => None,
                }
            }

            fn path(&self) -> $crate::instance::Instance {
                self.path.clone()
            }
        }

        impl $crate::netlist::NetlistMeta for $name<$crate::Search> {
            const MODULE_NAME: &'static str = $module;
            const FILE_PATH: &'static str = $file;

            const PORTS: &'static [$crate::netlist::PortSpec] = &[
                $($crate::netlist::PortSpec {
                    name: stringify!($input),
                    dir: $crate::netlist::PortDir::In,
                },)*
                $($crate::netlist::PortSpec {
                    name: stringify!($output),
                    dir: $crate::netlist::PortDir::Out,
                },)*
            ];
        }

        impl $crate::netlist::SearchableNetlist for $name<$crate::Search> {
            type Hit<'ctx> = $name<$crate::Match<'ctx>>;

            fn from_subgraph<'ctx>(
                m: &svql_subgraph::SubgraphIsomorphism<'ctx, 'ctx>,
                path: $crate::instance::Instance
            ) -> Self::Hit<'ctx> {
                $(
                    let $input = $crate::binding::bind_input(m, stringify!($input), 0);
                    let $input = $crate::Wire::with_val(
                        path.child(stringify!($input).to_string()),
                        $input
                    );
                )*
                $(
                    let $output = $crate::binding::bind_output(m, stringify!($output), 0);
                    let $output = $crate::Wire::with_val(
                        path.child(stringify!($output).to_string()),
                        $output
                    );
                )*

                Self::Hit::<'ctx> {
                    path: path.clone(),
                    $($input,)*
                    $($output,)*
                }
            }
        }
    };
}
