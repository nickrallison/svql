//! Primitive flip-flop definitions.
//!
//! This module provides specialized query components for various types of
//! flip-flops, including those with synchronous/asynchronous resets and
//! clock enables.

use crate::svql_common::{Config, ModuleConfig};
use crate::svql_driver::{Context, Driver, DriverKey};
use crate::svql_subgraph::cell::CellKind;
use crate::traits::{Component, PlannedQuery, Query, Reportable, Searchable};
use crate::{Instance, Match, Search, State, Wire};
use prjunnamed_netlist::Cell;
use std::sync::Arc;

macro_rules! impl_dff_primitive {
    ($name:ident, [$($port:ident),*], $filter:expr, $description:expr) => {
        #[doc = $description]
        #[derive(Clone, Debug)]
        pub struct $name<S: State> {
            /// The hierarchical path of this flip-flop instance.
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

        impl<S: State> Component<S> for $name<S> {
            fn path(&self) -> &Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
                if !path.starts_with(self.path()) {
                    return None;
                }
                self.find_port_inner(path.relative(self.path()))
            }

            fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
                let next = rel_path.first()?.as_ref();
                let tail = &rel_path[1..];
                match next {
                    $(stringify!($port) => self.$port.find_port_inner(tail),)*
                    _ => None,
                }
            }
        }

        impl Searchable for $name<Search> {
            fn instantiate(base_path: Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    $($port: Wire::new(base_path.child(stringify!($port)), ()),)*
                }
            }
        }

        impl<'a> Reportable for $name<Match<'a>> {
            fn to_report(&self, name: &str) -> crate::report::ReportNode {
                use crate::svql_subgraph::cell::SourceLocation;
                let source_loc = [$(self.$port.inner.get_source()),*]
                    .into_iter()
                    .flatten()
                    .next()
                    .unwrap_or_else(|| {
                        SourceLocation {
                            file: Arc::from(""),
                            lines: Vec::new()
                        }
                    });

                crate::report::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!($name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc,
                    children: Vec::new(),
                }
            }
        }

        impl Query for $name<Search> {
            type Matched<'a> = $name<Match<'a>>;

            fn query<'a>(
                &self,
                _driver: &Driver,
                context: &'a Context,
                key: &DriverKey,
                _config: &Config
            ) -> Vec<Self::Matched<'a>> {
                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                let matches: Vec<_> = index.cells_of_type_iter(CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .filter(|cell_wrapper| {
                        match cell_wrapper.get() {
                            Cell::Dff(ff) => {
                                let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                                check(ff)
                            }
                            _ => false,
                        }
                    })
                    .map(|cell| {
                        $name {
                            path: self.path.clone(),
                            $($port: Wire::new(self.$port.path.clone(), cell.clone())),*
                        }
                    })
                    .collect();
                matches
            }

            fn context(
                _driver: &Driver,
                _options: &ModuleConfig
            ) -> Result<Context, Box<dyn std::error::Error>> {
                Ok(Context::new())
            }
        }

        impl PlannedQuery for $name<Search> {
            fn expected_schema(&self) -> crate::ir::Schema {
                crate::ir::Schema {
                    columns: vec![$(stringify!($port).to_string()),*]
                }
            }

            fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
                let next = rel_path.first()?.as_ref();
                let mut column_idx = 0;
                $(
                    if next == stringify!($port) {
                        return Some(column_idx);
                    }
                    column_idx += 1;
                )*
                None
            }

            fn reconstruct<'a>(&self, cursor: &mut crate::ir::ResultCursor<'a>) -> Self::Matched<'a> {
                $name {
                    path: self.path.clone(),
                    $($port: Wire::new(self.$port.path.clone(), cursor.next_cell())),*
                }
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
