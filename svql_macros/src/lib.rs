// Procedural macros for SVQL pattern definitions.
//
// This crate provides three main derive macros for defining patterns:
// - `Netlist`: For netlist-based (Verilog) components
// - `Composite`: For composite patterns combining multiple components
// - `Variant`: For variant patterns (enums with multiple implementations)

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod composite;
mod netlist;
mod parsing;
mod variant;

/// Derive macro for defining netlist-based pattern components from Verilog files.
///
/// Automatically generates pattern matching code for a hardware module defined
/// in a Verilog file. Port declarations in the Verilog are mapped to struct fields.
///
/// # Attributes
///
/// - `#[netlist(file = \"path/to/file.v\", module = \"module_name\")]`: Specifies the Verilog file and module name
/// - `#[port(input)]` / `#[port(output)]`: Marks a field as an input or output port
/// - `#[port(input, rename = \"verilog_name\")]`: Renames the port when matching the Verilog file
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Netlist)]
/// #[netlist(file = "gates.v", module = "and_gate")]
/// pub struct AndGate {
///     #[port(input)]
///     pub a: Wire,
///     #[port(input)]
///     pub b: Wire,
///     #[port(output)]
///     pub y: Wire,
/// }
/// ```
#[proc_macro_derive(Netlist, attributes(netlist, port))]
#[proc_macro_error]
pub fn netlist(item: TokenStream) -> TokenStream {
    netlist::netlist_impl(item)
}

/// Derive macro for defining composite patterns that combine multiple components.
///
/// Allows defining complex patterns by specifying connections and aliases between
/// multiple pattern components (primitives or other composites).
///
/// # Attributes
///
/// - `#[submodule]`: Marks a field as a nested pattern component
/// - `#[connection(from = [...], to = [...])]`: Specifies connections between component ports
/// - `#[or_to(from = [...], to = [[...], [...]])]`: One-to-many connection pattern
/// - `#[alias(name, target = [...])]`: Creates an alias for a nested port
/// - `#[filter(condition = \"...\" )]`: Adds filtering constraints to the pattern
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Composite)]
/// #[connection(from = ["driver", "out"], to = ["receiver", "in"])]
/// pub struct MyComposite {
///     #[submodule]
///     pub driver: DriverComponent,
///     #[submodule]
///     pub receiver: ReceiverComponent,
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

/// Derive macro for defining variant patterns (enum-based pattern choices).
///
/// Allows defining multiple implementations of a pattern interface via an enum,
/// where each variant can have different port mappings.
///
/// # Attributes
///
/// - `#[variant_ports(...)]`: Declares the external ports of the variant pattern
/// - `#[map(...)]`: Maps variant variant implementation ports to the external ports
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Variant)]
/// #[variant_ports(input(a), input(b), output(y))]
/// pub enum DffAny {
///     #[map(a = ["a"], b = ["b"], y = ["y"])]
///     Basic(BasicDff),
///     #[map(a = ["clk"], b = ["d"], y = ["q"])]
///     WithEnable(DffWithEnable),
/// }
/// ```
#[proc_macro_derive(Variant, attributes(variant_ports, map))]
#[proc_macro_error]
pub fn variant(item: TokenStream) -> TokenStream {
    variant::variant_impl(item)
}
