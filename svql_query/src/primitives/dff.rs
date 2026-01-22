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

            // DataFrame API

            fn df_columns() -> &'static [::svql_query::session::ColumnDef] {
                static COLUMNS: &[::svql_query::session::ColumnDef] = &[
                    $(::svql_query::session::ColumnDef::wire(stringify!($port))),*
                ];
                COLUMNS
            }

            fn df_dependencies() -> &'static [::std::any::TypeId] {
                &[] // DFF primitives have no dependencies
            }

            fn df_register_search(registry: &mut ::svql_query::session::SearchRegistry) {
                use ::svql_query::session::{SearchFn, AnyTable};

                let search_fn: SearchFn = |ctx| {
                    let table = Self::df_search(ctx)?;
                    Ok(Box::new(table) as Box<dyn AnyTable>)
                };

                registry.register(
                    ::std::any::TypeId::of::<Self>(),
                    ::std::any::type_name::<Self>(),
                    Self::df_dependencies(),
                    search_fn,
                );
            }

            fn df_search(
                ctx: &::svql_query::session::ExecutionContext<'_>,
            ) -> Result<::svql_query::session::Table<Self>, ::svql_query::session::QueryError> {
                use ::svql_query::session::{TableBuilder, Row, QueryError, CellId};

                let driver = ctx.driver();
                let haystack_key = ctx.driver_key();

                // Get the haystack design
                let haystack_design = driver.get_design(&haystack_key)
                    .ok_or_else(|| QueryError::design_load(format!("Haystack design not found: {:?}", haystack_key)))?;
                let index = haystack_design.index();

                // Build the search instance at root
                let search_instance = Self::create_at(::svql_query::Instance::from_path(""));

                // Find matching cells
                let mut builder = TableBuilder::<Self>::new(Self::df_columns());

                for cell in index.cells_of_type_iter(::svql_query::CellKind::Dff).into_iter().flatten() {
                    let matches = match cell.get() {
                        prjunnamed_netlist::Cell::Dff(ff) => {
                            let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                            check(ff)
                        }
                        _ => false,
                    };

                    if matches {
                        let cell_id = CellId::new(cell.to_info().id as u32);
                        let row = Row::<Self>::new(builder.len() as u32, search_instance.path.to_string())
                            $(.with_wire(stringify!($port), Some(cell_id)))*;
                        builder.push(row);
                    }
                }

                builder.build()
            }

            fn df_rehydrate(
                row: &::svql_query::session::Row<Self>,
                _store: &::svql_query::session::Store,
            ) -> Option<Self::Match> {
                let path = ::svql_query::Instance::from_path(row.path());
                // For DFF primitives, all ports map to the same cell
                // (the DFF cell itself - we store the cell ID in each wire column)
                Some($name {
                    path: path.clone(),
                    $(
                        $port: {
                            let cell_opt = row.wire(stringify!($port)).map(|c| {
                                ::svql_query::CellInfo {
                                    id: c.cell_idx() as usize,
                                    kind: ::svql_query::CellKind::Dff,
                                    source_loc: None,
                                }
                            });
                            ::svql_query::Wire::new(path.child(stringify!($port)), cell_opt)
                        }
                    ),*
                })
            }
        }

        impl ::svql_query::traits::MatchedComponent for $name<::svql_query::Match> {
            type Search = $name<::svql_query::Search>;
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
