//! Primitive flip-flop definitions.
//!
//! This module provides specialized query components for various types of
//! flip-flops, including those with synchronous/asynchronous resets and
//! clock enables.

use crate::State;

#[macro_export]
macro_rules! impl_dff_primitive {
    ($name:ident, [$($port:ident),*], $filter:expr, $description:expr) => {
        #[doc = $description]
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct $name<S: State> {
            /// The hierarchical path of this flip-flop instance.
            pub path: ::svql_query::Instance,
            $(
                #[doc = concat!("The ", stringify!($port), " port wire.")]
                pub $port: ::svql_query::Wire<S>
            ),*
        }

        impl<S: State> $name<S> {
            $(
                #[doc = concat!("Returns a reference to the ", stringify!($port), " port.")]
                pub fn $port(&self) -> Option<&::svql_query::Wire<S>> {
                    Some(&self.$port)
                }
            )*
        }

        impl<S: State> ::svql_query::Hardware for $name<S> {
            type State = S;

            fn path(&self) -> &::svql_query::Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            fn children(&self) -> Vec<&dyn ::svql_query::Hardware<State = Self::State>> {
                vec![ $( &self.$port ),* ]
            }

            fn find_port(&self, path: &::svql_query::Instance) -> Option<&::svql_query::Wire<S>> {
                if !path.starts_with(self.path()) {
                    return None;
                }
                let rel_path = path.relative(self.path());
                let next = rel_path.first()?.as_ref();
                match next {
                    $(stringify!($port) => self.$port.find_port(path),)*
                    _ => None,
                }
            }

            fn report(&self, name: &str) -> ::svql_query::ReportNode {
                let source_loc = [$(self.$port.source()),*]
                    .into_iter()
                    .flatten()
                    .next();

                ::svql_query::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!($name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc,
                    children: Vec::new(),
                }
            }
        }

        impl ::svql_query::traits::SearchableComponent for $name<::svql_query::Search> {
            type Kind = ::svql_query::kind::Netlist;
            type Match = $name<::svql_query::Match>;

            fn create_at(base_path: ::svql_query::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    $($port: ::svql_query::Wire::new(base_path.child(stringify!($port)), ()),)*
                }
            }

            fn build_context(
                _driver: &::svql_driver::Driver,
                _options: &::svql_common::ModuleConfig
            ) -> Result<::svql_query::Context, Box<dyn std::error::Error>> {
                Ok(::svql_query::Context::new())
            }

            fn execute_search(
                &self,
                _driver: &::svql_driver::Driver,
                context: &::svql_query::Context,
                key: &::svql_driver::DriverKey,
                _config: &::svql_common::Config
            ) -> Vec<Self::Match> {
                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                let matches: Vec<_> = index.cells_of_type_iter(::svql_query::CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .filter(|cell_wrapper| {
                        match cell_wrapper.get() {
                            prjunnamed_netlist::Cell::Dff(ff) => {
                                let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                                check(ff)
                            }
                            _ => false,
                        }
                    })
                    .map(|cell| {
                        $name {
                            path: self.path.clone(),
                            $($port: ::svql_query::Wire::new(self.$port.path.clone(), Some(cell.to_info()))),*
                        }
                    })
                    .collect();
                matches
            }
        }

        impl ::svql_query::traits::MatchedComponent for $name<::svql_query::Match> {
            type Search = $name<::svql_query::Search>;
        }

        impl ::svql_query::session::Dehydrate for $name<::svql_query::Match> {
            const SCHEMA: ::svql_query::session::QuerySchema = ::svql_query::session::QuerySchema::new(
                stringify!($name),
                &[
                    $(::svql_query::session::WireFieldDesc { name: stringify!($port) }),*
                ],
                &[],
            );

            fn dehydrate(&self) -> ::svql_query::session::DehydratedRow {
                let mut row = ::svql_query::session::DehydratedRow::new(self.path.to_string());
                $(
                    row = row.with_wire(stringify!($port), self.$port.inner.as_ref().map(|c| c.id as u32));
                )*
                row
            }
        }

        impl ::svql_query::session::Rehydrate for $name<::svql_query::Match> {
            const TYPE_NAME: &'static str = stringify!($name);

            fn rehydrate(
                row: &::svql_query::session::MatchRow,
                ctx: &::svql_query::session::RehydrateContext<'_>,
            ) -> Result<Self, ::svql_query::session::SessionError> {
                let path = ::svql_query::Instance::from_path(&row.path);
                Ok(Self {
                    path: path.clone(),
                    $(
                        $port: ctx.rehydrate_wire(path.child(stringify!($port)), row.wire(stringify!($port))),
                    )*
                })
            }
        }

        impl ::svql_query::session::SearchDehydrate for $name<::svql_query::Search> {
            const MATCH_SCHEMA: ::svql_query::session::QuerySchema =
                <$name<::svql_query::Match> as ::svql_query::session::Dehydrate>::SCHEMA;

            fn execute_dehydrated(
                &self,
                _driver: &::svql_driver::Driver,
                context: &::svql_query::Context,
                key: &::svql_driver::DriverKey,
                _config: &::svql_common::Config,
                results: &mut ::svql_query::session::DehydratedResults,
            ) -> Vec<u32> {
                // Register our schema using full type path
                let type_key = Self::type_key();
                results.register_schema(type_key, &Self::MATCH_SCHEMA);

                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                index.cells_of_type_iter(::svql_query::CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .filter(|cell_wrapper| {
                        match cell_wrapper.get() {
                            prjunnamed_netlist::Cell::Dff(ff) => {
                                let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                                check(ff)
                            }
                            _ => false,
                        }
                    })
                    .map(|cell| {
                        let cell_id = cell.to_info().id as u32;
                        let row = ::svql_query::session::DehydratedRow::new(self.path.to_string())
                            $(.with_wire(stringify!($port), Some(cell_id)))*;
                        results.push(type_key, row)
                    })
                    .collect()
            }
        }
    };
}

impl_dff_primitive!(
    DffAny,
    [clk, d, en, q],
    |_| true,
    "Matches any flip-flop cell regardless of reset or enable configuration."
);

impl_dff_primitive!(
    Sdffe,
    [clk, d, reset, en, q],
    |ff| ff.has_reset() && ff.has_enable(),
    "Matches flip-flops with synchronous reset and clock enable."
);

impl_dff_primitive!(
    Adffe,
    [clk, d, reset_n, en, q],
    |ff| ff.has_clear() && ff.has_enable(),
    "Matches flip-flops with asynchronous reset (clear) and clock enable."
);

impl_dff_primitive!(
    Sdff,
    [clk, d, reset, q],
    |ff| ff.has_reset() && !ff.has_enable(),
    "Matches flip-flops with synchronous reset and no clock enable."
);

impl_dff_primitive!(
    Adff,
    [clk, d, reset_n, q],
    |ff| ff.has_clear() && !ff.has_enable(),
    "Matches flip-flops with asynchronous reset (clear) and no clock enable."
);

impl_dff_primitive!(
    Dffe,
    [clk, d, en, q],
    |ff| !ff.has_reset() && !ff.has_clear() && ff.has_enable(),
    "Matches flip-flops with clock enable and no reset logic."
);

impl_dff_primitive!(
    Dff,
    [clk, d, q],
    |ff| !ff.has_reset() && !ff.has_clear() && !ff.has_enable(),
    "Matches basic flip-flops with no reset or clock enable logic."
);
