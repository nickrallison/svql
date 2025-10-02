// svql_query/src/composite/composite_macro.rs
//
// Declarative macros for simplifying composite and enum_composite query definitions.
// These generate boilerplate impls for WithPath, Composite, SearchableComposite/EnumComposite, etc.
// Assumes sub-patterns (e.g., AndGate) implement SearchableNetlist.
// Supports parallel querying via #[cfg(feature = "parallel")] (add `use std::thread;` and `use tracing::{event, Level};` in usage files).

/// Macro for defining composite queries (structural patterns with sub-netlists and required connections).
///
/// Usage in a query file (e.g., src/queries/composite/dff_then_and.rs):
/// ```rust
/// #[cfg(feature = "parallel")]
/// use std::thread;
/// use tracing::{event, Level};
///
/// use crate::composite::{composite, Composite};  // Import macro and traits
/// use crate::queries::netlist::basic::{dff::Sdffe, and::AndGate};
///
/// composite! {
///     name: SdffeThenAnd,
///     subs: [ sdffe: Sdffe, and_gate: AndGate ],
///     connections: [
///         sdffe . q => and_gate . a,
///         sdffe . q => and_gate . b
///     ]
/// }
/// ```
///
/// - Generates: Struct `SdffeThenAnd<S>`, `new(path)`, `WithPath`, `Composite` (with connections in one group),
///   `MatchedComposite`, and `SearchableComposite` (with merged context and iproduct!-based query + validation).
/// - Parallel: Conditionally spawns threads per sub-query if "parallel" feature enabled.
/// - Connections: All in a single validation group (must have at least one valid connection).
/// - Query: Runs sub-queries (parallel or sequential), uses `iproduct!` to combine, filters via `validate_connections`.
/// - Empty connections: Allowed (uses `vec![vec![]]`—validation always passes).
/// - Limitations: Up to ~10 subs (due to `iproduct!` tuple limits); one connection group.
/// - Discovery: build.rs regexes detect the generated `impl SearchableComposite`.
#[macro_export]
macro_rules! composite {
    // Main variant with connections
    (
        name: $name:ident,
        subs: [ $( $sub_name:ident : $sub_type:ty ),* $(,)? ],
        connections: [ $( $from_sub:ident . $from_port:ident => $to_sub:ident . $to_port:ident ),* $(,)? ]
    ) => {
        composite!(@internal_impl_with_conn $name, $( $sub_name : $sub_type ),* , $( $from_sub . $from_port => $to_sub . $to_port ),* );
    };
    // Empty connections variant
    (
        name: $name:ident,
        subs: [ $( $sub_name:ident : $sub_type:ty ),* $(,)? ],
        connections: []
    ) => {
        composite!(@internal_impl_no_conn $name, $( $sub_name : $sub_type ),* );
    };
    // Internal: WITH connections (direct recursive call, no module prefix)
    (@internal_impl_with_conn $name:ident, $( $sub_name:ident : $sub_type:ty ),* , $( $from_sub:ident . $from_port:ident => $to_sub:ident . $to_port:ident ),* ) => {
        #[derive(Debug, Clone)]
        pub struct $name<S>
        where
            S: $crate::State,
        {
            pub path: $crate::instance::Instance,
            $( pub $sub_name: $sub_type<S>, )*
        }

        impl<S> $name<S>
        where
            S: $crate::State,
        {
            pub fn new(path: $crate::instance::Instance) -> Self {
                #[allow(unused_variables)]
                Self {
                    path: path.clone(),
                    $( $sub_name: <$sub_type<S>>::new(path.child(stringify!($sub_name).to_string())), )*
                }
            }
        }

        impl<S> $crate::WithPath<S> for $name<S>
        where
            S: $crate::State,
        {
            fn find_port(&self, p: &$crate::instance::Instance) -> Option<&$crate::Wire<S>> {
                let idx = self.path.height() + 1;
                match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
                    $( Some(stringify!($sub_name)) => self.$sub_name.find_port(p), )*
                    _ => None,
                }
            }
            fn path(&self) -> $crate::instance::Instance {
                self.path.clone()
            }
        }

        impl<S> $crate::composite::Composite<S> for $name<S>
        where
            S: $crate::State,
        {
            fn connections(&self) -> Vec<Vec<$crate::Connection<S>>> {
                vec![vec![
                    $( $crate::Connection {
                        from: self.$from_sub.$from_port.clone(),
                        to: self.$to_sub.$to_port.clone(),
                    }, )*
                ]]
            }
        }

        impl<'ctx> $crate::composite::MatchedComposite<'ctx> for $name<$crate::Match<'ctx>> {}

        impl $crate::composite::SearchableComposite for $name<$crate::Search> {
            type Hit<'ctx> = $name<$crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::context::Context, Box<dyn std::error::Error>> {
                let mut merged = None;
                $(
                    let this_ctx = <$sub_type::<$crate::Search>>::context(driver, config)?;
                    merged = match merged {
                        None => Some(this_ctx),
                        Some(prev) => Some(prev.merge(this_ctx)),
                    };
                )*
                #[allow(unused_variables)]  // For 0 subs
                { merged.ok_or_else(|| "No sub-patterns in composite".into()) }
            }

            fn query<'ctx>(
                haystack_key: &svql_driver::DriverKey,
                context: &'ctx svql_driver::context::Context,
                path: $crate::instance::Instance,
                config: &svql_common::Config,
            ) -> Vec<Self::Hit<'ctx>> {
                #[cfg(feature = "parallel")]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing with parallel queries", stringify!($name));
                    let tuple_return = std::thread::scope(|scope| {
                        $( let $sub_name _thread = scope.spawn(|| {
                            <$sub_type::<$crate::Search>>::query(
                                haystack_key,
                                context,
                                path.child(stringify!($sub_name).to_string()),
                                config,
                            )
                        }); )*
                        (
                            $( $sub_name _thread.join().expect(concat!("Failed to join ", stringify!($sub_name), " thread")), )*
                        )
                    });
                    let ( $( ref $sub_name _joined, )* ) = tuple_return;
                    $crate::itertools::iproduct!($( $sub_name _joined.iter().cloned() ),* )
                        .map(|( $( ref $sub_name ),* )| $name::<$crate::Match<'ctx>> {
                            path: path.clone(),
                            $( $sub_name: $sub_name .clone(), )*
                        })
                        .filter(|hit| hit.validate_connections(hit.connections()))
                        .collect()
                }

                #[cfg(not(feature = "parallel"))]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing sequential queries", stringify!($name));
                    $( let $sub_name _matches = <$sub_type::<$crate::Search>>::query(
                        haystack_key,
                        context,
                        path.child(stringify!($sub_name).to_string()),
                        config,
                    ); )*
                    $crate::itertools::iproduct!($( $sub_name _matches ),* )
                        .map(|( $( $sub_name ),* )| $name::<$crate::Match<'ctx>> {
                            path: path.clone(),
                            $( $sub_name ),*
                        })
                        .filter(|hit| hit.validate_connections(hit.connections()))
                        .collect()
                }
            }
        }
    };
    // Internal: WITHOUT connections (direct recursive call)
    (@internal_impl_no_conn $name:ident, $( $sub_name:ident : $sub_type:ty ),* ) => {
        #[derive(Debug, Clone)]
        pub struct $name<S>
        where
            S: $crate::State,
        {
            pub path: $crate::instance::Instance,
            $( pub $sub_name: $sub_type<S>, )*
        }

        impl<S> $name<S>
        where
            S: $crate::State,
        {
            pub fn new(path: $crate::instance::Instance) -> Self {
                #[allow(unused_variables)]
                Self {
                    path: path.clone(),
                    $( $sub_name: <$sub_type<S>>::new(path.child(stringify!($sub_name).to_string())), )*
                }
            }
        }

        impl<S> $crate::WithPath<S> for $name<S>
        where
            S: $crate::State,
        {
            fn find_port(&self, p: &$crate::instance::Instance) -> Option<&$crate::Wire<S>> {
                let idx = self.path.height() + 1;
                match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
                    $( Some(stringify!($sub_name)) => self.$sub_name.find_port(p), )*
                    _ => None,
                }
            }
            fn path(&self) -> $crate::instance::Instance {
                self.path.clone()
            }
        }

        impl<S> $crate::composite::Composite<S> for $name<S>
        where
            S: $crate::State,
        {
            fn connections(&self) -> Vec<Vec<$crate::Connection<S>>> {
                vec![vec![]]
            }
        }

        impl<'ctx> $crate::composite::MatchedComposite<'ctx> for $name<$crate::Match<'ctx>> {}

        impl $crate::composite::SearchableComposite for $name<$crate::Search> {
            type Hit<'ctx> = $name<$crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::context::Context, Box<dyn std::error::Error>> {
                let mut merged = None;
                $(
                    let this_ctx = <$sub_type::<$crate::Search>>::context(driver, config)?;
                    merged = match merged {
                        None => Some(this_ctx),
                        Some(prev) => Some(prev.merge(this_ctx)),
                    };
                )*
                #[allow(unused_variables)]  // For 0 subs
                { merged.ok_or_else(|| "No sub-patterns in composite".into()) }
            }

            fn query<'ctx>(
                haystack_key: &svql_driver::DriverKey,
                context: &'ctx svql_driver::context::Context,
                path: $crate::instance::Instance,
                config: &svql_common::Config,
            ) -> Vec<Self::Hit<'ctx>> {
                #[cfg(feature = "parallel")]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing with parallel queries", stringify!($name));
                    let tuple_return = std::thread::scope(|scope| {
                        $( let $sub_name _thread = scope.spawn(|| {
                            <$sub_type::<$crate::Search>>::query(
                                haystack_key,
                                context,
                                path.child(stringify!($sub_name).to_string()),
                                config,
                            )
                        }); )*
                        (
                            $( $sub_name _thread.join().expect(concat!("Failed to join ", stringify!($sub_name), " thread")), )*
                        )
                    });
                    let ( $( ref $sub_name _joined, )* ) = tuple_return;
                    $crate::itertools::iproduct!($( $sub_name _joined.iter().cloned() ),* )
                        .map(|( $( ref $sub_name ),* )| $name::<$crate::Match<'ctx>> {
                            path: path.clone(),
                            $( $sub_name: $sub_name .clone(), )*
                        })
                        .filter(|hit| hit.validate_connections(hit.connections()))
                        .collect()
                }

                #[cfg(not(feature = "parallel"))]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing sequential queries", stringify!($name));
                    $( let $sub_name _matches = <$sub_type::<$crate::Search>>::query(
                        haystack_key,
                        context,
                        path.child(stringify!($sub_name).to_string()),
                        config,
                    ); )*
                    $crate::itertools::iproduct!($( $sub_name _matches ),* )
                        .map(|( $( $sub_name ),* )| $name::<$crate::Match<'ctx>> {
                            path: path.clone(),
                            $( $sub_name ),*
                        })
                        .filter(|hit| hit.validate_connections(hit.connections()))
                        .collect()
                }
            }
        }
    };
}

/// Macro for defining enum_composite queries (disjoint variants over sub-netlists).
///
/// Usage in a query file (e.g., src/queries/enum_composite/and_any.rs):
/// ```rust
/// #[cfg(feature = "parallel")]
/// use std::thread;
/// use tracing::{event, Level};
///
/// use crate::composite::{enum_composite, EnumComposite};  // Import macro and traits
/// use crate::queries::netlist::basic::and::{AndGate, AndMux, AndNor};
///
/// enum_composite! {
///     name: AndAny,
///     variants: [
///         Gate ( "and_gate" ) : AndGate,
///         Mux  ( "and_mux" ) : AndMux,
///         Nor  ( "and_nor" ) : AndNor
///     ]
/// }
/// ```
///
/// - Generates: Enum `AndAny<S>` with variants, `WithPath` (delegates to inner), `EnumComposite`,
///   `MatchedEnumComposite`, and `SearchableEnumComposite` (merged context, chained queries).
/// - Parallel: Conditionally spawns threads per variant if "parallel" feature enabled.
/// - Variants: Each needs a literal instance name (e.g., `"and_gate"`) for `path.child(inst_name)`.
/// - Query: Runs sub-queries (parallel or sequential), maps to enum variants, chains results (no validation/connections).
/// - Discovery: build.rs regexes detect the generated `impl SearchableEnumComposite`.
#[macro_export]
macro_rules! enum_composite {
    (
        name: $name:ident,
        variants: [ $( $var:ident ( $inst_name:literal ) : $sub_type:ty ),* $(,)? ]
    ) => {
        #[derive(Debug, Clone)]
        pub enum $name<S>
        where
            S: $crate::State,
        {
            $( $var( $sub_type::<S> ), )*
        }

        impl<S> $crate::WithPath<S> for $name<S>
        where
            S: $crate::State,
        {
            fn find_port(&self, p: &$crate::instance::Instance) -> Option<&$crate::Wire<S>> {
                match self {
                    $( $name::$var(inner) => inner.find_port(p), )*
                }
            }
            fn path(&self) -> $crate::instance::Instance {
                match self {
                    $( $name::$var(inner) => inner.path(), )*
                }
            }
        }

        impl<S> $crate::composite::EnumComposite<S> for $name<S> where S: $crate::State {}

        impl<'ctx> $crate::composite::MatchedEnumComposite<'ctx> for $name<$crate::Match<'ctx>> {}

        impl $crate::composite::SearchableEnumComposite for $name<$crate::Search> {
            type Hit<'ctx> = $name<$crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::context::Context, Box<dyn std::error::Error>> {
                let mut merged = None;
                $(
                    let this_ctx = <$sub_type::<$crate::Search>>::context(driver, config)?;
                    merged = match merged {
                        None => Some(this_ctx),
                        Some(prev) => Some(prev.merge(this_ctx)),
                    };
                )*
                #[allow(unused_variables)]  // For 0 variants
                { merged.ok_or_else(|| "No variants in enum_composite".into()) }
            }

            fn query<'ctx>(
                haystack_key: &svql_driver::DriverKey,
                context: &'ctx svql_driver::context::Context,
                path: $crate::instance::Instance,
                config: &svql_common::Config,
            ) -> Vec<Self::Hit<'ctx>> {
                #[cfg(feature = "parallel")]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing with parallel queries", stringify!($name));
                    let tuple_return = std::thread::scope(|scope| {
                        $( let $var _thread = scope.spawn(|| {
                            <$sub_type::<$crate::Search>>::query(
                                haystack_key,
                                context,
                                path.child($inst_name.to_string()),
                                config,
                            )
                        }); )*
                        (
                            $( $var _thread.join().expect(concat!("Failed to join ", stringify!($var), " thread")), )*
                        )
                    });
                    let ( $( ref $var _joined, )* ) = tuple_return;
                    let mut hits = Vec::new();
                    $(
                        hits.extend($var _joined.iter().cloned().map(|inner| $name::<$crate::Match<'ctx>>::$var(inner)));
                    )*
                    hits
                }

                #[cfg(not(feature = "parallel"))]
                {
                    tracing::event!(tracing::Level::INFO, "{}::query: executing sequential queries", stringify!($name));
                    $( let $var _matches = <$sub_type::<$crate::Search>>::query(
                        haystack_key,
                        context,
                        path.child($inst_name.to_string()),
                        config,
                    ); )*
                    let mut hits = Vec::new();
                    $(
                        hits.extend($var _matches.into_iter().map(|inner| $name::<$crate::Match<'ctx>>::$var(inner)));
                    )*
                    hits
                }
            }
        }
    };
}
