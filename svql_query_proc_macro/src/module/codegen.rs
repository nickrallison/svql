use proc_macro2::TokenStream;

use crate::lower::Ir;

pub fn codegen(_ir: Ir) -> TokenStream {
    TokenStream::new()
}
