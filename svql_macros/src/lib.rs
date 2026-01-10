use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod common;
mod composite;
mod netlist;
mod variant;

/// Defines a Netlist query component.
///
/// Usage:
/// ```rust,ignore
/// #[netlist(file = "path/to/file.v", name = "module_name")]
/// pub struct MyNetlist<S: State> {
///     clk: Wire<S>,
///     #[rename("reset")]
///     rst: Wire<S>,
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn netlist(args: TokenStream, input: TokenStream) -> TokenStream {
    netlist::netlist_impl(args, input)
}

/// Defines a Composite query component.
///
/// Usage:
/// ```rust,ignore
/// #[composite]
/// pub struct MyComposite<S: State> {
///     #[submodule]
///     sub: SubQuery<S>,
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn composite(args: TokenStream, input: TokenStream) -> TokenStream {
    composite::composite_impl(args, input)
}

/// Defines a Variant (Enum) query component.
///
/// Usage:
/// ```rust,ignore
/// #[variant(ports(clk, enable, reset))]
/// pub enum MyVariant<S: State> {
///     #[variant(map(enable = "we", reset = "rst"))]
///     TypeA(TypeA<S>),
///     #[variant(map(enable = "en", reset = None))]
///     TypeB(TypeB<S>),
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn variant(args: TokenStream, input: TokenStream) -> TokenStream {
    variant::variant_impl(args, input)
}
