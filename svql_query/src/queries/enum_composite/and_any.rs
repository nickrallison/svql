// use crate::{
//     Match, Search, State, Wire, WithPath,
//     composite::{EnumComposite, MatchedEnumComposite, SearchableEnumComposite},
//     instance::Instance,
//     netlist::SearchableNetlist,
//     queries::netlist::basic::and::{AndGate, AndMux, AndNor},
// };
// use svql_common::{Config, ModuleConfig};
// use svql_driver::{Context, Driver, DriverKey};

// #[cfg(feature = "parallel")]
// use std::thread;

// #[derive(Debug, Clone)]
// pub enum AndAny<S>
// where
//     S: State,
// {
//     Gate(AndGate<S>),
//     Mux(AndMux<S>),
//     Nor(AndNor<S>),
// }

// impl<S> WithPath<S> for AndAny<S>
// where
//     S: State,
// {
//     fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
//         match self {
//             AndAny::Gate(inner) => inner.find_port(p),
//             AndAny::Mux(inner) => inner.find_port(p),
//             AndAny::Nor(inner) => inner.find_port(p),
//         }
//     }

//     fn path(&self) -> Instance {
//         match self {
//             AndAny::Gate(inner) => inner.path(),
//             AndAny::Mux(inner) => inner.path(),
//             AndAny::Nor(inner) => inner.path(),
//         }
//     }
// }

// impl<S> EnumComposite<S> for AndAny<S> where S: State {}

// impl<'ctx> MatchedEnumComposite<'ctx> for AndAny<Match<'ctx>> {}

// impl SearchableEnumComposite for AndAny<Search> {
//     type Hit<'ctx> = AndAny<Match<'ctx>>;

//     fn context(
//         driver: &Driver,
//         config: &ModuleConfig,
//     ) -> Result<Context, Box<dyn std::error::Error>> {
//         let and_gate_context = AndGate::<Search>::context(driver, config)?;
//         let and_mux_context = AndMux::<Search>::context(driver, config)?;
//         let and_nor_context = AndNor::<Search>::context(driver, config)?;

//         Ok(and_gate_context
//             .merge(and_mux_context)
//             .merge(and_nor_context))
//     }

//     fn query<'ctx>(
//         haystack_key: &DriverKey,
//         context: &'ctx Context,
//         path: Instance,
//         config: &Config,
//     ) -> Vec<Self::Hit<'ctx>> {
//         #[cfg(feature = "parallel")]
//         let (and_gate_matches, and_mux_matches, and_nor_matches) = {
//             tracing::event!(
//                 tracing::Level::INFO,
//                 "AndAny::query: executing with parallel queries"
//             );

//             std::thread::scope(|scope| {
//                 let and_gate_thread = scope.spawn(|| {
//                     AndGate::<Search>::query(
//                         haystack_key,
//                         context,
//                         path.child("and_gate".to_string()),
//                         config,
//                     )
//                 });

//                 let and_mux_thread = scope.spawn(|| {
//                     AndMux::<Search>::query(
//                         haystack_key,
//                         context,
//                         path.child("and_mux".to_string()),
//                         config,
//                     )
//                 });

//                 let and_nor_thread = scope.spawn(|| {
//                     AndNor::<Search>::query(
//                         haystack_key,
//                         context,
//                         path.child("and_nor".to_string()),
//                         config,
//                     )
//                 });

//                 (
//                     and_gate_thread
//                         .join()
//                         .expect("Failed to join and_gate thread"),
//                     and_mux_thread
//                         .join()
//                         .expect("Failed to join and_mux thread"),
//                     and_nor_thread
//                         .join()
//                         .expect("Failed to join and_nor thread"),
//                 )
//             })
//         };

//         #[cfg(not(feature = "parallel"))]
//         let (and_gate_matches, and_mux_matches, and_nor_matches) = {
//             tracing::event!(
//                 tracing::Level::INFO,
//                 "AndAny::query: executing sequential queries"
//             );

//             (
//                 AndGate::<Search>::query(
//                     haystack_key,
//                     context,
//                     path.child("and_gate".to_string()),
//                     config,
//                 ),
//                 AndMux::<Search>::query(
//                     haystack_key,
//                     context,
//                     path.child("and_mux".to_string()),
//                     config,
//                 ),
//                 AndNor::<Search>::query(
//                     haystack_key,
//                     context,
//                     path.child("and_nor".to_string()),
//                     config,
//                 ),
//             )
//         };

//         and_gate_matches
//             .into_iter()
//             .map(AndAny::<Match<'ctx>>::Gate)
//             .chain(and_mux_matches.into_iter().map(AndAny::<Match<'ctx>>::Mux))
//             .chain(and_nor_matches.into_iter().map(AndAny::<Match<'ctx>>::Nor))
//             .collect()
//     }
// }

// // svql_query/src/queries/enum_composite/and_any.rs
// //
// // Enum composite query: Any of AndGate, AndMux, or AndNor.
// // Generated via enum_composite! macro.

// // use crate::{
// //     enum_composite,
// //     queries::netlist::basic::and::{AndGate, AndMux, AndNor},
// // };

// // // Generate everything: enum, impls, query, etc.
// // enum_composite! {
// //     name: AndAny,
// //     variants: [
// //         Gate   ( "and_gate" ) : AndGate,
// //         Mux    ( "and_mux"  ) : AndMux,
// //         Nor    ( "and_nor"  ) : AndNor
// //     ]
// // }

// // // Optional: Overrides similar to above (e.g., for parallel).

use svql_macros::enum_composite;

enum_composite! {
    name: AndAny,
    variants: [
        (Gate, "and_gate", AndGate),
        (Mux, "and_mux",  AndMux),
        (Nor, "and_nor",  AndNor),
    ]
}
