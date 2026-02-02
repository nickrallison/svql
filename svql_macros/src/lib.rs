// svql_macros/src/lib.rs

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod composite;
mod netlist;
mod parsing;
mod variant;

/// Derive macro for netlist-based pattern components.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, Netlist)]
/// #[netlist(file = "path/to/file.v", module = "module_name")]
/// struct MyGate {
///     #[port(input)]
///     a: Wire,
///     #[port(output)]
///     y: Wire,
///     #[port(input, rename = "in_data")]
///     data: Wire,
/// }
/// ```
#[proc_macro_derive(Netlist, attributes(netlist, port))]
#[proc_macro_error]
pub fn netlist(item: TokenStream) -> TokenStream {
    netlist::netlist_impl(item)
}

/// Derive macro for composite pattern components.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, Composite)]
/// #[connection(from = ["driver", "out"], to = ["receiver", "in"])]
/// #[or_to(from = ["and1", "y"], to = [["and2", "a"], ["and2", "b"]])]
/// struct MyComposite {
///     #[submodule]
///     driver: Driver,
///     #[submodule]
///     receiver: Receiver,
///     #[alias(input, target = ["driver", "in"])]
///     input: Wire,
/// }
/// ```
#[proc_macro_derive(
    Composite,
    attributes(submodule, alias, connection, or_to, or_from, or_group, filter)
)]
#[proc_macro_error]
pub fn composite(item: TokenStream) -> TokenStream {
    composite::composite_impl(item)
}

/// Derive macro for variant (enum) pattern components.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, Variant)]
/// #[variant_ports(input(a), input(b), output(y))]
/// enum DffAny {
///     #[map(a = ["a"], b = ["b"], y = ["y"])]
///     Basic(Dff),
///     #[map(a = ["clk"], b = ["d"], y = ["q"])]
///     WithEnable(Dffe),
/// }
/// ```
#[proc_macro_derive(Variant, attributes(variant_ports, map))]
#[proc_macro_error]
pub fn variant(item: TokenStream) -> TokenStream {
    variant::variant_impl(item)
}
