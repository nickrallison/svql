use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, GenericArgument, Lit, Meta, PathArguments, Token, Type};

#[allow(dead_code)]
pub fn get_attribute_value(attrs: &[Attribute], attr_name: &str, key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
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

/// Robustly replaces the generic argument corresponding to `State` with `::svql_query::Search`.
///
/// This function assumes the standard SVQL pattern where the `State` parameter is
/// a Type argument (not a lifetime or const). It replaces the *first* Type argument found.
///
/// Examples:
/// - `MyQuery<S>` -> `MyQuery<::svql_query::Search>`
/// - `MyQuery<'a, S>` -> `MyQuery<'a, ::svql_query::Search>`
/// - `MyQuery<S, const N: usize>` -> `MyQuery<::svql_query::Search, const N: usize>`
pub fn replace_state_generic(ty: &Type) -> TokenStream {
    if let Type::Path(type_path) = ty {
        let mut new_path = type_path.clone();

        // We modify the last segment of the path (e.g., `MyQuery` in `crate::module::MyQuery<S>`)
        if let Some(last_segment) = new_path.path.segments.last_mut() {
            if let PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
                // Iterate over the generic arguments (Lifetimes, Types, Consts, Bindings, Constraints)
                for arg in &mut args.args {
                    if let GenericArgument::Type(_) = arg {
                        // Replace the first Type argument we encounter with `Search`
                        *arg = GenericArgument::Type(syn::parse_quote!(::svql_query::Search));

                        // We break immediately to avoid replacing subsequent Type arguments
                        // (if the struct has multiple generic types).
                        break;
                    }
                }
            }
        }
        quote! { #new_path }
    } else {
        // If the type is not a Path (e.g., it's a reference `&T`, array `[T]`, etc.),
        // we return it unmodified. The generated code will likely fail to compile
        // if this type is used as a submodule, which is the intended behavior
        // (submodules must be named types).
        quote! { #ty }
    }
}

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
