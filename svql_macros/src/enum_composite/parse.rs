use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result, Token, bracketed, parse2, punctuated::Punctuated};

#[derive(Clone)]
pub struct Variant {
    pub variant_name: Ident,
    pub inst_name: LitStr,
    pub ty: syn::Type,
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
        syn::bracketed!(variants_content in input); // Outer brackets for the array
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
        let inst_content;
        bracketed!(inst_content in input); // Brackets around LitStr
        let inst_name = inst_content.parse::<LitStr>()?; // Parse LitStr inside brackets
        input.parse::<Token![:]>()?; // Colon after brackets
        let ty = input.parse::<syn::Type>()?;
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
    use super::*;
    use quote::quote;

    #[test]
    fn valid_syntax() {
        let ts = quote! {
            name: AndAny,
            variants: [
                (Gate, "and_gate", AndGate)
                (Mux, "and_mux",  AndMux),
                (Nor, "and_nor",  AndNor)
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.name.to_string(), "AndAny");
        assert_eq!(ast.variants.len(), 3);
        assert_eq!(ast.variants[0].variant_name.to_string(), "Gate");
        assert_eq!(ast.variants[0].inst_name.value(), "and_gate");
        assert_eq!(ast.variants[1].variant_name.to_string(), "Mux");
        assert_eq!(ast.variants[1].inst_name.value(), "and_mux");
        assert_eq!(ast.variants[2].variant_name.to_string(), "Nor");
        assert_eq!(ast.variants[2].inst_name.value(), "and_nor");
    }

    #[test]
    fn valid_syntax_single_variant() {
        let ts = quote! {
            name: Single,
            variants: [
                Only [ "only" ] : Type
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.variants.len(), 1);
        assert_eq!(ast.variants[0].inst_name.value(), "only");
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

    #[test]
    fn valid_syntax_with_trailing_comma() {
        let ts = quote! {
            name: Trailing,
            variants: [
                One [ "one" ] : Type,
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.variants.len(), 1);
    }
}
