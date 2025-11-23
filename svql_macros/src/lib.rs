//! Procedural macros for SVQL.
//!
//! This crate defines macros for defining netlists, composites, and enum composites
//! to simplify SVQL query construction.

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod composite;
mod variant;
mod netlist;

#[proc_macro]
#[proc_macro_error]
pub fn netlist(input: TokenStream) -> TokenStream {
    netlist::netlist_inner(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn composite(input: TokenStream) -> TokenStream {
    composite::composite_inner(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn variant(input: TokenStream) -> TokenStream {
    variant::variant_inner(input)
}
