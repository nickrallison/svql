//! Primitive hardware gate definitions.
//!
//! This module provides standard logic and arithmetic gates used as the
//! atomic building blocks for structural queries.

use crate::common::{Config, ModuleConfig};
use crate::driver::{Context, Driver, DriverKey};
use crate::session::{DehydratedResults, DehydratedRow, SearchDehydrate};
use crate::subgraph::cell::CellKind;
use crate::traits::{Hardware, MatchedComponent, SearchableComponent, kind};
use crate::{Instance, Match, ReportNode, Search, State, Wire};

macro_rules! define_primitive_gate {
    (
        $name:ident,
        $kind:ident,
        [$($port:ident),*]
    ) => {
        #[doc = concat!("A primitive ", stringify!($kind), " gate component.")]
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct $name<S: State> {
            /// The hierarchical path of this gate instance.
            pub path: Instance,
            $(
                #[doc = concat!("The ", stringify!($port), " port wire.")]
                pub $port: Wire<S>
            ),*
        }

        impl<S: State> $name<S> {
            $(
                #[doc = concat!("Returns a reference to the ", stringify!($port), " port.")]
                pub fn $port(&self) -> Option<&Wire<S>> {
                    Some(&self.$port)
                }
            )*
        }

        impl<S: State> Hardware for $name<S> {
            type State = S;

            fn path(&self) -> &Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
                vec![ $( &self.$port ),* ]
            }

            fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
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

            fn report(&self, name: &str) -> ReportNode {
                let source_loc = [$(self.$port.source()),*]
                    .into_iter()
                    .flatten()
                    .next();

                ReportNode {
                    name: name.to_string(),
                    type_name: stringify!($name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc,
                    children: Vec::new(),
                }
            }
        }

        impl SearchableComponent for $name<Search> {
            type Kind = kind::Netlist;
            type Match = $name<Match>;

            fn create_at(base_path: Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    $($port: Wire::new(base_path.child(stringify!($port)), ()),)*
                }
            }

            fn build_context(
                _driver: &Driver,
                _config: &ModuleConfig
            ) -> Result<Context, Box<dyn std::error::Error>> {
                Ok(Context::new())
            }

            /// Scans the design index for all cells matching the primitive type.
            fn execute_search(
                &self,
                _driver: &Driver,
                context: &Context,
                key: &DriverKey,
                config: &Config
            ) -> Vec<Self::Match> {
                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                match config.dedupe {
                    crate::common::Dedupe::All => {
                        /* All Cells Deduplicated */
                    }
                    _ => {
                        if config.dedupe != crate::common::Dedupe::None {
                            crate::tracing::warn!(
                                "{} deduplication strategy {:?} is not yet implemented for primitive cell scans. Returning all matches.",
                                self.type_name(),
                                config.dedupe
                            );
                        }
                    }
                }

                index.cells_of_type_iter(CellKind::$kind)
                    .into_iter()
                    .flatten()
                    .map(|cell| {
                        $name {
                            path: self.path.clone(),
                            $($port: Wire::new(self.$port.path.clone(), Some(cell.to_info()))),*
                        }
                    })
                    .collect()

            }

            // DataFrame API

            fn df_columns() -> &'static [crate::session::ColumnDef] {
                static COLUMNS: &[crate::session::ColumnDef] = &[
                    $(crate::session::ColumnDef::wire(stringify!($port))),*
                ];
                COLUMNS
            }

            fn df_dependencies() -> &'static [::std::any::TypeId] {
                &[] // Primitive gates have no dependencies
            }

            fn df_register_search(registry: &mut crate::session::SearchRegistry) {
                use crate::session::{SearchFn, AnyTable};

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
                ctx: &crate::session::ExecutionContext<'_>,
            ) -> Result<crate::session::Table<Self>, crate::session::QueryError> {
                use crate::session::{TableBuilder, Row, QueryError, CellId};

                let driver = ctx.driver();
                let haystack_key = ctx.driver_key();

                // Get the haystack design
                let haystack_design = driver.get_design(&haystack_key)
                    .ok_or_else(|| QueryError::design_load(format!("Haystack design not found: {:?}", haystack_key)))?;
                let index = haystack_design.index();

                // Build the search instance at root
                let search_instance = Self::create_at(Instance::from_path(""));

                // Find matching cells
                let mut builder = TableBuilder::<Self>::new(Self::df_columns());

                for cell in index.cells_of_type_iter(CellKind::$kind).into_iter().flatten() {
                    let cell_id = CellId::new(cell.to_info().id as u32);
                    let row = Row::<Self>::new(builder.len() as u32, search_instance.path.to_string())
                        $(.with_wire(stringify!($port), Some(cell_id)))*;
                    builder.push(row);
                }

                builder.build()
            }

            fn df_rehydrate(
                row: &crate::session::Row<Self>,
                _store: &crate::session::Store,
            ) -> Option<Self::Match> {
                let path = Instance::from_path(row.path());
                // For primitive gates, all ports map to the same cell
                Some($name {
                    path: path.clone(),
                    $(
                        $port: {
                            let cell_opt = row.wire(stringify!($port)).map(|c| {
                                crate::subgraph::cell::CellInfo {
                                    id: c.cell_idx() as usize,
                                    kind: CellKind::$kind,
                                    source_loc: None,
                                }
                            });
                            Wire::new(path.child(stringify!($port)), cell_opt)
                        }
                    ),*
                })
            }
        }

        impl MatchedComponent for $name<Match> {
            type Search = $name<Search>;
        }

        impl crate::session::Dehydrate for $name<Match> {
            const SCHEMA: crate::session::QuerySchema = crate::session::QuerySchema::new(
                stringify!($name),
                &[
                    $(crate::session::WireFieldDesc { name: stringify!($port) }),*
                ],
                &[],
            );

            fn dehydrate(&self) -> crate::session::DehydratedRow {
                let mut row = crate::session::DehydratedRow::new(self.path.to_string());
                $(
                    row = row.with_wire(stringify!($port), self.$port.inner.as_ref().map(|c| c.id as u32));
                )*
                row
            }
        }

        impl crate::session::Rehydrate for $name<Match> {
            const TYPE_NAME: &'static str = stringify!($name);

            fn rehydrate(
                row: &crate::session::MatchRow,
                ctx: &crate::session::RehydrateContext<'_>,
            ) -> Result<Self, crate::session::SessionError> {
                let path = Instance::from_path(&row.path);
                Ok(Self {
                    path: path.clone(),
                    $(
                        $port: ctx.rehydrate_wire(path.child(stringify!($port)), row.wire(stringify!($port))),
                    )*
                })
            }
        }

        impl SearchDehydrate for $name<Search> {
            const MATCH_SCHEMA: crate::session::QuerySchema = <$name<Match> as crate::session::Dehydrate>::SCHEMA;

            fn execute_dehydrated(
                &self,
                _driver: &Driver,
                context: &Context,
                key: &DriverKey,
                config: &Config,
                results: &mut DehydratedResults,
            ) -> Vec<u32> {
                // Register our schema using full type path
                let type_key = Self::type_key();
                results.register_schema(type_key, &Self::MATCH_SCHEMA);

                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                match config.dedupe {
                    crate::common::Dedupe::All => { /* All Cells Deduplicated */ }
                    _ => {
                        if config.dedupe != crate::common::Dedupe::None {
                            crate::tracing::warn!(
                                "{} deduplication strategy {:?} is not yet implemented for primitive cell scans. Returning all matches.",
                                self.type_name(),
                                config.dedupe
                            );
                        }
                    }
                }

                index.cells_of_type_iter(CellKind::$kind)
                    .into_iter()
                    .flatten()
                    .map(|cell| {
                        let cell_id = cell.to_info().id as u32;
                        let row = DehydratedRow::new(self.path.to_string())
                            $(.with_wire(stringify!($port), Some(cell_id)))*;
                        results.push(type_key, row)
                    })
                    .collect()
            }
        }
    };
}

define_primitive_gate!(AndGate, And, [a, b, y]);
define_primitive_gate!(OrGate, Or, [a, b, y]);
define_primitive_gate!(NotGate, Not, [a, y]);
define_primitive_gate!(BufGate, Buf, [a, y]);
define_primitive_gate!(XorGate, Xor, [a, b, y]);
define_primitive_gate!(MuxGate, Mux, [a, b, sel, y]);

define_primitive_gate!(EqGate, Eq, [a, b, y]);
define_primitive_gate!(LtGate, ULt, [a, b, y]);
define_primitive_gate!(AddGate, Adc, [a, b, y]);
define_primitive_gate!(MulGate, Mul, [a, b, y]);
