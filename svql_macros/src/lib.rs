use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod composite;
mod netlist;
mod variant;

#[proc_macro_derive(Netlist, attributes(netlist, rename))]
#[proc_macro_error]
pub fn netlist(item: TokenStream) -> TokenStream {
    netlist::netlist_impl(item)
}

#[proc_macro_derive(Composite, attributes(submodule, path))]
#[proc_macro_error]
pub fn composite(item: TokenStream) -> TokenStream {
    composite::composite_impl(item)
}

#[proc_macro_derive(Variant, attributes(variant, ports))]
#[proc_macro_error]
pub fn variant(item: TokenStream) -> TokenStream {
    variant::variant_impl(item)
}
