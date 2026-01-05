use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, GenericArgument, GenericParam, Lit, Meta, PathArguments, Token, Type,
    WherePredicate,
};

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
                        break;
                    }
                }
            }
        }
        quote! { #new_path }
    } else {
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

/// Removes the first Type generic parameter (assumed to be State) from the generics list.
/// This is used to create the `impl` generics for concrete Search/Match implementations.
pub fn remove_state_generic(generics: &syn::Generics) -> syn::Generics {
    let mut new_generics = generics.clone();
    let mut removed_ident = None;

    let mut new_params = Punctuated::new();
    let mut found = false;

    for param in new_generics.params {
        if !found {
            if let GenericParam::Type(type_param) = &param {
                removed_ident = Some(type_param.ident.clone());
                found = true;
                continue;
            }
        }
        new_params.push(param);
    }
    new_generics.params = new_params;

    // Clean up where clause if it references the removed generic
    if let Some(ident) = removed_ident {
        if let Some(where_clause) = &mut new_generics.where_clause {
            let mut new_predicates = Punctuated::new();
            for pred in &where_clause.predicates {
                let keep = match pred {
                    WherePredicate::Type(pt) => {
                        if let Type::Path(tp) = &pt.bounded_ty {
                            !tp.path.is_ident(&ident)
                        } else {
                            true
                        }
                    }
                    _ => true,
                };
                if keep {
                    new_predicates.push(pred.clone());
                }
            }
            where_clause.predicates = new_predicates;
        }
    }

    new_generics
}

/// Constructs the type identifier with the State generic replaced by a concrete type (Search or Match).
/// e.g., `MyQuery<S, T>` -> `MyQuery<::svql_query::Search, T>`
pub fn make_replaced_type(
    ident: &syn::Ident,
    generics: &syn::Generics,
    replacement: TokenStream,
) -> TokenStream {
    let mut args = Punctuated::<TokenStream, Token![,]>::new();
    let mut found = false;

    for param in &generics.params {
        match param {
            GenericParam::Type(t) => {
                if !found {
                    args.push(replacement.clone());
                    found = true;
                } else {
                    let i = &t.ident;
                    args.push(quote! { #i });
                }
            }
            GenericParam::Const(c) => {
                let i = &c.ident;
                args.push(quote! { #i });
            }
            GenericParam::Lifetime(l) => {
                let i = &l.lifetime;
                args.push(quote! { #i });
            }
        }
    }

    if args.is_empty() {
        // Should not happen if we are replacing a state generic, but for safety
        quote! { #ident }
    } else {
        quote! { #ident < #args > }
    }
}
