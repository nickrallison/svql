use proc_macro2::TokenStream;
use syn::{Result, parse2};
use syn::parse::{Parse, ParseStream};
use proc_macro_error::abort;


pub struct Ast { }

impl Parse for Ast {
    fn parse(_stream: ParseStream) -> Result<Self> {
        Ok(Ast { })
    }
}

pub fn parse(ts: TokenStream) -> Ast {
    match parse2::<Ast>(ts) {
        Ok(ast) => ast,
        Err(e) => {
            abort!(e.span(), e)
        },
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;
    #[test]
    fn valid_syntax() {
        parse(
            quote!(
            ),
        );
    }
}
