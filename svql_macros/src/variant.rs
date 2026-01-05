use crate::common;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, Fields, ItemEnum, Lit, Meta, Token, parse_macro_input};

struct VariantInfo {
    ident: syn::Ident,
    ty: syn::Type,
    port_map: std::collections::HashMap<String, Option<String>>,
}

pub fn variant_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_enum = parse_macro_input!(input as ItemEnum);
    let enum_name = &item_enum.ident;
    let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();
    let generics = &item_enum.generics;

    // --- Parsing Phase ---

    // 1. Parse common ports from #[variant(ports(...))]
    let mut common_ports = Vec::new();
    let args_parser = Punctuated::<Meta, Token![,]>::parse_terminated;

    if let Ok(parsed_args) = args_parser.parse(args) {
        for meta in parsed_args {
            if let Meta::List(list) = meta {
                if list.path.is_ident("ports") {
                    let parser = Punctuated::<syn::Ident, Token![,]>::parse_terminated;
                    if let Ok(nested) = list.parse_args_with(parser) {
                        for ident in nested {
                            common_ports.push(ident);
                        }
                    }
                }
            }
        }
    }

    // 2. Parse Variants and Maps
    let mut variants_info = Vec::new();

    for variant in &item_enum.variants {
        let ident = variant.ident.clone();

        // Extract the inner type (e.g., AndGate<S> from Gate(AndGate<S>))
        let ty = if let Fields::Unnamed(ref fields) = variant.fields {
            if let Some(field) = fields.unnamed.first() {
                field.ty.clone()
            } else {
                panic!("Variant {} must have exactly one unnamed field", ident);
            }
        } else {
            panic!("Variant {} must be tuple style (e.g., Name(Type))", ident);
        };

        let mut port_map = std::collections::HashMap::new();

        // Non-destructive attribute parsing
        for attr in &variant.attrs {
            if attr.path().is_ident("variant") {
                if let Ok(list) =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    for meta in list {
                        if let Meta::List(map_list) = meta {
                            if map_list.path.is_ident("map") {
                                if let Ok(map_items) = map_list.parse_args_with(
                                    Punctuated::<Meta, Token![,]>::parse_terminated,
                                ) {
                                    for map_item in map_items {
                                        if let Meta::NameValue(nv) = map_item {
                                            let key = nv.path.get_ident().unwrap().to_string();
                                            if let Expr::Lit(expr_lit) = nv.value {
                                                if let Lit::Str(val) = expr_lit.lit {
                                                    port_map.insert(key, Some(val.value()));
                                                }
                                            } else if let Expr::Path(p) = nv.value {
                                                if p.path.is_ident("None") {
                                                    port_map.insert(key, None);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        variants_info.push(VariantInfo {
            ident,
            ty,
            port_map,
        });
    }

    // --- Generation Phase ---

    let abstract_ident = format_ident!("__Abstract");

    // 1. Enum Definition
    // We reconstruct the enum variants to strip attributes like #[variant(map...)]
    let variant_defs = variants_info.iter().map(|v| {
        let ident = &v.ident;
        let ty = &v.ty;
        quote! { #ident(#ty) }
    });

    let abstract_fields = common_ports.iter().map(|p| {
        quote! { #p: ::svql_query::Wire<S> }
    });

    let expanded_enum = quote! {
        #[derive(Clone, Debug)]
        pub enum #enum_name #generics {
            #(#variant_defs),*,
            #[doc(hidden)]
            #abstract_ident {
                path: ::svql_query::instance::Instance,
                #(#abstract_fields),*
            }
        }
    };

    // 2. Accessors for Common Ports
    let accessors = common_ports.iter().map(|port| {
        let port_str = port.to_string();
        let arms = variants_info.iter().map(|v| {
            let v_ident = &v.ident;
            if let Some(Some(mapped_field)) = v.port_map.get(&port_str) {
                let mapped_ident = format_ident!("{}", mapped_field);
                quote! { Self::#v_ident(inner) => Some(&inner.#mapped_ident) }
            } else {
                quote! { Self::#v_ident(_) => None }
            }
        });

        quote! {
            pub fn #port(&self) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#arms),*,
                    Self::#abstract_ident { #port, .. } => Some(#port),
                }
            }
        }
    });

    // 3. Component Implementation
    let path_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! { Self::#ident(inner) => inner.path() }
    });

    let find_port_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! { Self::#ident(inner) => inner.find_port(path) }
    });

    let find_port_inner_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! { Self::#ident(inner) => inner.find_port_inner(rel_path) }
    });

    // 4. Searchable Implementation (Instantiate)
    let abstract_init = common_ports.iter().map(|p| {
        let p_str = p.to_string();
        quote! { #p: ::svql_query::Wire::new(base_path.child(#p_str), ()) }
    });

    // 5. Query Implementation
    let query_blocks = variants_info.iter().map(|v| {
        let v_ident = &v.ident;
        let v_ty = &v.ty;
        // Use the robust generic replacement
        let search_type = common::replace_state_generic(v_ty);

        quote! {
            {
                // Instantiate the specific variant in Search mode
                let sub_query = <#search_type as ::svql_query::traits::Searchable>::instantiate(
                    ::svql_query::traits::Component::path(self).clone()
                );
                let results = sub_query.query(driver, context, key, config);

                // Map results back to the Enum variant
                all_results.extend(
                    results.into_iter().map(#enum_name::<::svql_query::Match>::#v_ident)
                );
            }
        }
    });

    let context_merges = variants_info.iter().map(|v| {
        let v_ty = &v.ty;
        let search_type = common::replace_state_generic(v_ty);
        quote! {
            let sub_ctx = <#search_type>::context(driver, options)?;
            ctx = ctx.merge(sub_ctx);
        }
    });

    // 6. Reportable Implementation
    let report_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! {
            Self::#ident(inner) => {
                let mut node = inner.to_report(name);
                node.details = Some(stringify!(#ident).to_string());
                node
            }
        }
    });

    let expanded = quote! {
        #expanded_enum

        impl #impl_generics #enum_name #ty_generics #where_clause {
            #(#accessors)*
        }

        impl #impl_generics ::svql_query::traits::Projected for #enum_name<::svql_query::Search> #where_clause {
            type Pattern = #enum_name<::svql_query::Search>;
            type Result = #enum_name<::svql_query::Match>;
        }

        impl #impl_generics ::svql_query::traits::Projected for #enum_name<::svql_query::Match> #where_clause {
            type Pattern = #enum_name<::svql_query::Search>;
            type Result = #enum_name<::svql_query::Match>;
        }

        impl #impl_generics ::svql_query::traits::Component<S> for #enum_name #ty_generics #where_clause {
            fn path(&self) -> &::svql_query::instance::Instance {
                match self {
                    #(#path_arms),*,
                    Self::#abstract_ident { path, .. } => path,
                }
            }

            fn type_name(&self) -> &'static str {
                stringify!(#enum_name)
            }

            fn find_port(&self, path: &::svql_query::instance::Instance) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#find_port_arms),*,
                    Self::#abstract_ident { .. } => None,
                }
            }

            fn find_port_inner(&self, rel_path: &[std::sync::Arc<str>]) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#find_port_inner_arms),*,
                    Self::#abstract_ident { .. } => None,
                }
            }
        }

        impl ::svql_query::traits::Searchable for #enum_name<::svql_query::Search> {
            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self::#abstract_ident {
                    path: base_path.clone(),
                    #(#abstract_init),*
                }
            }
        }

        impl<'a> ::svql_query::traits::Reportable for #enum_name<::svql_query::Match> {
            fn to_report(&self, name: &str) -> ::svql_query::report::ReportNode {
                use ::svql_query::subgraph::cell::SourceLocation;

                match self {
                    #(#report_arms),*,
                    Self::#abstract_ident { .. } => panic!("__Abstract variant found in Match state during reporting. This indicates a bug in the query execution logic."),
                }
            }
        }

        impl ::svql_query::traits::Query for #enum_name<::svql_query::Search> {
            fn query<'a>(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &'a ::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Result> {
                use ::svql_query::traits::{Component, Searchable};
                ::svql_query::tracing::info!("{} searching variants", self.log_label());

                let mut all_results = Vec::new();
                #(#query_blocks)*

                ::svql_query::tracing::info!("{} found {} total matches across variants", self.log_label(), all_results.len());
                all_results
            }

            fn context(
                driver: &::svql_query::driver::Driver,
                options: &::svql_query::common::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::driver::Context::new();
                #(#context_merges)*
                Ok(ctx)
            }
        }
    };

    TokenStream::from(expanded)
}
