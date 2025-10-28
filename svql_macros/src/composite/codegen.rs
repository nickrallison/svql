use proc_macro2::TokenStream;

use super::lower::Ir;

pub fn codegen(_ir: Ir) -> TokenStream {
    TokenStream::new()
}
