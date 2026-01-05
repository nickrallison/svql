//! Primitive hardware gate definitions.
//!
//! This module provides standard logic and arithmetic gates used as the
//! atomic building blocks for structural queries.

use crate::svql_common::{Config, ModuleConfig};
use crate::svql_driver::{Context, Driver, DriverKey};
use crate::svql_subgraph::cell::CellKind;
use crate::traits::{Component, Query, Reportable, Searchable};
use crate::{Instance, Match, Search, State, Wire};
use std::sync::Arc;

macro_rules! define_primitive_gate {
    (
        $name:ident,
        $kind:ident,
        [$($port:ident),*]
    ) => {
        #[doc = concat!("A primitive ", stringify!($kind), " gate component.")]
        #[derive(Clone, Debug)]
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
                let rel_path = path.relative(self.path());
                self.find_port_inner(rel_path)
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

        impl<'a> Reportable for $name<Match> {
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
            type Matched<'a> = $name<Match>;

            /// Scans the design index for all cells matching the primitive type.
            fn query<'a>(
                &self,
                _driver: &Driver,
                context: &'a Context,
                key: &DriverKey,
                config: &Config
            ) -> Vec<Self::Matched<'a>> {
                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                match config.dedupe {
                    crate::svql_common::Dedupe::All => {
                        /* All Cells Deduplicated */
                    }
                    _ => {
                        if config.dedupe != crate::svql_common::Dedupe::None {
                            crate::tracing::warn!(
                                "{} deduplication strategy {:?} is not yet implemented for primitive cell scans. Returning all matches.",
                                self.log_label(),
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
                            $($port: Wire::new(self.$port.path.clone(), cell.clone())),*
                        }
                    })
                    .collect()
            }

            fn context(
                _driver: &Driver,
                _config: &ModuleConfig
            ) -> Result<Context, Box<dyn std::error::Error>> {
                Ok(Context::new())
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
