// svql_macros/src/enum_composites/parse.rs
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result, Token, bracketed, parenthesized, parse2, punctuated::Punctuated};

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
        bracketed!(variants_content in input); // Outer brackets for the array
        let variants_punctuated: Punctuated<Variant, Token![,]> =
            variants_content.parse_terminated(Variant::parse, Token![,])?;
        let variants = variants_punctuated.into_iter().collect();

        Ok(Ast { name, variants })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input); // Parenthesized tuple for the variant
        let variant_name = content.parse::<Ident>()?; // First: Ident (variant name)
        content.parse::<Token![,]>()?; // Comma after variant name
        let inst_name = content.parse::<LitStr>()?; // Second: LitStr (inst name)
        content.parse::<Token![,]>()?; // Comma after inst name
        let ty = content.parse::<syn::Type>()?; // Third: Type
        // Closing paren is handled by parenthesized!
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

    // Subtest 1: Parse a single Variant in isolation (focuses on tuple internals)
    #[test]
    fn test_parse_single_variant() {
        // Test a single tuple: (Gate, "and_gate", AndGate)
        let ts = quote! { (Gate, "and_gate", AndGate) };
        let variant: Variant = parse2(ts).unwrap();
        assert_eq!(variant.variant_name.to_string(), "Gate");
        assert_eq!(variant.inst_name.value(), "and_gate");

        // Verify Type is a Path (simple ident)
        if let syn::Type::Path(ty_path) = variant.ty {
            assert_eq!(ty_path.path.segments[0].ident.to_string(), "AndGate");
        } else {
            panic!("Expected Type::Path for AndGate");
        }

        // Test with spaces (should still parse)
        let ts_with_space = quote! { ( Mux , "and_mux" , AndMux ) };
        let variant: Variant = parse2(ts_with_space).unwrap();
        assert_eq!(variant.variant_name.to_string(), "Mux");
        assert_eq!(variant.inst_name.value(), "and_mux");
    }

    // Subtest 2: Parse variants array indirectly via full Ast (focuses on Punctuated and commas)
    // (Avoids direct stream manipulation to sidestep version issues)
    #[test]
    fn test_parse_variants_punctuated_indirectly() {
        // Test bracketed content with Punctuated: [ (Gate,...), (Mux,...) ]
        // Note: Commas between variants are REQUIRED for Punctuated
        let ts = quote! {
            name: Dummy,
            variants: [
                (Gate, "and_gate", AndGate),
                (Mux, "and_mux", AndMux)
            ]
        };
        let ast = parse(ts);
        let variants = &ast.variants;
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].variant_name.to_string(), "Gate");
        assert_eq!(variants[0].inst_name.value(), "and_gate");
        assert_eq!(variants[1].variant_name.to_string(), "Mux");
        assert_eq!(variants[1].inst_name.value(), "and_mux");

        // Test empty array via full Ast
        let empty_ts = quote! {
            name: EmptyDummy,
            variants: []
        };
        let empty_ast = parse(empty_ts);
        assert_eq!(empty_ast.variants.len(), 0);

        // Test trailing comma in array via full Ast
        let trailing_ts = quote! {
            name: TrailingDummy,
            variants: [
                (Nor, "and_nor", AndNor),
            ]
        };
        let trailing_ast = parse(trailing_ts);
        let trailing_variants = &trailing_ast.variants;
        assert_eq!(trailing_variants.len(), 1);
        assert_eq!(trailing_variants[0].inst_name.value(), "and_nor");
    }

    // Subtest 3: Parse full Ast (your original syntax, but with required commas between variants)
    #[test]
    fn test_parse_full_ast() {
        // FIXED: Added commas after each tuple (required for Punctuated)
        let ts = quote! {
            name: AndAny,
            variants: [
                (Gate, "and_gate", AndGate),
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

    // Subtest 4: Parse empty variants (full Ast)
    #[test]
    fn test_parse_empty_variants() {
        let ts = quote! {
            name: Empty,
            variants: []
        };
        let ast = parse(ts);
        assert_eq!(ast.name.to_string(), "Empty");
        assert_eq!(ast.variants.len(), 0);
    }

    // Subtest 5: Parse with trailing comma (full Ast)
    #[test]
    fn test_parse_trailing_comma() {
        let ts = quote! {
            name: Trailing,
            variants: [
                (One, "one", Type),
            ]
        };
        let ast = parse(ts);
        assert_eq!(ast.name.to_string(), "Trailing");
        assert_eq!(ast.variants.len(), 1);
        assert_eq!(ast.variants[0].variant_name.to_string(), "One");
        assert_eq!(ast.variants[0].inst_name.value(), "one");
    }

    // Subtest 6: Error case - missing comma inside tuple (should abort)
    #[test]
    #[should_panic(expected = "expected `,`")] // Syn error for missing comma in tuple
    fn test_parse_error_missing_comma_in_tuple() {
        let ts = quote! { (Gate "and_gate" AndGate) }; // Missing commas
        let _variant: Variant = parse2(ts).unwrap(); // Should fail and panic via abort
    }

    // Subtest 7: Error case - invalid Type (e.g., non-Type token; should abort)
    #[test]
    #[should_panic(
        expected = "expected one of: `for`"
    )] // Matches Syn 2.0's verbose type error (substring for robustness)
    fn test_parse_error_invalid_type() {
        let ts = quote! { (Gate, "and_gate", 123) }; // 123 is not a Type
        let _variant: Variant = parse2(ts).unwrap(); // Should fail and panic via abort
    }

    // Subtest 8: Parse multiple variants with trailing comma (should fail without fix, pass with robust parsing)
    #[test]
    fn test_parse_multiple_variants_trailing_comma() {
        let ts = quote! {
            name: AndAnyTrailing,
            variants: [
                (Gate, "and_gate", AndGate),
                (Mux, "and_mux", AndMux),
                (Nor, "and_nor", AndNor),
            ]
        };
        // This should parse successfully if trailing comma is handled for multi-items
        let ast = parse(ts);
        assert_eq!(ast.name.to_string(), "AndAnyTrailing");
        assert_eq!(ast.variants.len(), 3);
        assert_eq!(ast.variants[2].inst_name.value(), "and_nor");
    }
}
