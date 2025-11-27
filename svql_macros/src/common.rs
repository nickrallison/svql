use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, GenericArgument, Lit, Meta, PathArguments, Token, Type};

#[allow(dead_code)]
pub fn get_attribute_value(attrs: &[Attribute], attr_name: &str, key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            // Parse #[attr_name(key = "value", ...)]
            if let Ok(nested) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            {
                for meta in nested {
                    if let Meta::NameValue(nv) = meta {
                        if nv.path.is_ident(key) {
                            if let Expr::Lit(expr_lit) = nv.value {
                                if let Lit::Str(lit_str) = expr_lit.lit {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn has_attribute(attrs: &[Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attr_name))
}

/// Extracts the value of a specific key from a comma-separated list of key-value pairs in attributes.
/// e.g. #[netlist(file = "foo.v", name = "bar")] -> get_arg(..., "file") returns Some("foo.v")
pub fn parse_args_map(args: proc_macro::TokenStream) -> std::collections::HashMap<String, String> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed_args = parser.parse(args).expect("Failed to parse macro arguments");

    let mut map = std::collections::HashMap::new();
    for meta in parsed_args {
        if let Meta::NameValue(nv) = meta {
            if let Some(ident) = nv.path.get_ident() {
                let key = ident.to_string();
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(lit) = expr_lit.lit {
                        map.insert(key, lit.value());
                    }
                }
            }
        }
    }
    map
}

/// Helper to replace generic <S> with <::svql_query::Search> in a type.
/// This is crucial for instantiating sub-queries in the Search state.
pub fn replace_generic_with_search(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Path(type_path) = ty {
        let mut new_path = type_path.path.clone();
        if let Some(last_segment) = new_path.segments.last_mut() {
            // Assume the last segment has the generic <S>
            last_segment.arguments =
                PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Default::default(),
                    args: {
                        let mut args = Punctuated::new();
                        args.push(GenericArgument::Type(syn::parse_quote!(
                            ::svql_query::Search
                        )));
                        args
                    },
                    gt_token: Default::default(),
                });
        }
        quote! { #new_path }
    } else {
        // Fallback: just quote the type (might fail if it depends on S)
        quote! { #ty }
    }
}
