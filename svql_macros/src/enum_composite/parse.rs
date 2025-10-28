use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result, Token, bracketed, parse2, punctuated::Punctuated, Type};

#[derive(Clone)]
pub struct Variant {
    pub variant_name: Ident,
    pub inst_name: LitStr,
    pub ty: Type,
}

pub struct Ast {
    pub name: Ident,
    pub variants: Vec<Variant>,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse "name: Identifier"
        let name_kw: Ident = input.parse()?;
        if name_kw != "name" {
            return Err(input.error("expected 'name'"));
        }
        input.parse::<Token![:]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        // Parse "variants: [ ... ]"
        let variants_kw: Ident = input.parse()?;
        if variants_kw != "variants" {
            return Err(input.error("expected 'variants'"));
        }
        input.parse::<Token![:]>()?;
        let variants_content;
        bracketed!(variants_content in input);
        let variants_punctuated: Punctuated<Variant, Token![,]> =
            variants_content.parse_terminated(Variant::parse, Token![,])?;
        let variants = variants_punctuated.into_iter().collect();

        // Optional trailing comma after variants array
        let _ = input.parse::<Token![,]>();

        Ok(Ast { name, variants })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let variant_name = input.parse::<Ident>()?;
        input.parse::<Token![(]>()?;  // FIXED: Use Token![(] for opening parenthesis
        let inst_name = input.parse::<LitStr>()?;
        input.parse::<Token![)]>()?;  // FIXED: Use Token![)] for closing parenthesis
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        Ok(Variant {
            variant_name,
            inst_name,
            ty,
        })
    }
}

pub fn parse(ts: TokenStream) -> Ast {
    match parse2::<Ast>(ts) {
        Ok(ast) => ast,
        Err(e) => abort!(e.span(), e),
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use super::*;

    #[test]
    fn valid_syntax() {
        let ts = quote! {
            name: AndAny,
            variants: [
                Gate ( "and_gate" ) : AndGate,
                Mux  ( "and_mux" ) : AndMux,
                Nor  ( "and_nor" ) : AndNor
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.name.to_string(), "AndAny");
        assert_eq!(ast.variants.len(), 3);
        assert_eq!(ast.variants[0].variant_name.to_string(), "Gate");
        assert_eq!(ast.variants[0].inst_name.value(), "and_gate");
    }

    #[test]
    fn valid_syntax_single_variant() {
        let ts = quote! {
            name: Single,
            variants: [
                Only ( "only" ) : Type
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.variants.len(), 1);
    }

    #[test]
    fn valid_syntax_empty_variants() {
        let ts = quote! {
            name: Empty,
            variants: []
        };
        let ast = parse(ts);
        assert_eq!(ast.variants.len(), 0);
    }
}