use crate::common;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Expr, Fields, ItemEnum, Lit, Meta, Token, parse_macro_input};

pub fn variant_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_enum = parse_macro_input!(input as ItemEnum);
    let enum_name = &item_enum.ident;
    let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();
    let generics = &item_enum.generics;

    let mut common_ports = Vec::new();
    let args_parser = Punctuated::<Meta, Token![,]>::parse_terminated;

    use syn::parse::Parser;
    if let Ok(parsed_args) = args_parser.parse(args) {
        for meta in parsed_args {
            if let Meta::List(list) = meta {
                if list.path.is_ident("ports") {
                    let parser = Punctuated::<syn::Ident, Token![,]>::parse_terminated;
                    let nested: Punctuated<syn::Ident, Token![,]> = list
                        .parse_args_with(parser)
                        .expect("ports must be comma-separated idents");
                    for ident in nested {
                        common_ports.push(ident);
                    }
                }
            }
        }
    }

    let mut variant_names = Vec::new();
    let mut variant_types = Vec::new();
    let mut variant_maps = Vec::new();

    for variant in item_enum.variants.iter_mut() {
        variant_names.push(variant.ident.clone());

        if let Fields::Unnamed(ref fields) = variant.fields {
            if let Some(field) = fields.unnamed.first() {
                variant_types.push(field.ty.clone());
            } else {
                panic!("Variant must have one unnamed field");
            }
        } else {
            panic!("Variant must be tuple style");
        }

        let mut map = std::collections::HashMap::new();
        let mut attrs_to_remove = Vec::new();

        for (i, attr) in variant.attrs.iter().enumerate() {
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
                                                    map.insert(key, Some(val.value()));
                                                }
                                            } else if let Expr::Path(p) = nv.value {
                                                if p.path.is_ident("None") {
                                                    map.insert(key, None);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                attrs_to_remove.push(i);
            }
        }
        for i in attrs_to_remove.into_iter().rev() {
            variant.attrs.remove(i);
        }
        variant_maps.push(map);
    }

    let abstract_ident = format_ident!("__Abstract");
    let port_fields = common_ports.iter().map(|p| {
        quote! { #p: ::svql_query::Wire<S> }
    });

    let variants_code = item_enum.variants.iter().map(|v| quote! { #v });

    let expanded_enum = quote! {
        #[derive(Clone, Debug)]
        pub enum #enum_name #generics {
            #(#variants_code),*,
            #[doc(hidden)]
            #abstract_ident {
                path: ::svql_query::instance::Instance,
                #(#port_fields),*
            }
        }
    };

    let accessors = common_ports.iter().map(|port| {
        let port_str = port.to_string();
        let arms = variant_names
            .iter()
            .zip(variant_maps.iter())
            .map(|(v_name, v_map)| {
                if let Some(Some(mapped_field)) = v_map.get(&port_str) {
                    let mapped_ident = format_ident!("{}", mapped_field);
                    quote! { Self::#v_name(inner) => Some(&inner.#mapped_ident) }
                } else {
                    quote! { Self::#v_name(_) => None }
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

    let component_arms_path: Vec<_> = variant_names
        .iter()
        .map(|v| quote! { Self::#v(inner) => inner.path() })
        .collect();
    let _component_arms_children: Vec<_> = variant_names
        .iter()
        .map(|v| quote! { Self::#v(inner) => inner.children() })
        .collect();
    let component_arms_find_port: Vec<_> = variant_names
        .iter()
        .map(|v| quote! { Self::#v(inner) => inner.find_port(path) })
        .collect();
    let component_arms_find_port_inner: Vec<_> = variant_names
        .iter()
        .map(|v| quote! { Self::#v(inner) => inner.find_port_inner(rel_path) })
        .collect();

    let abstract_init_fields = common_ports.iter().map(|p| {
        let p_str = p.to_string();
        quote! { #p: ::svql_query::Wire::new(base_path.child(#p_str), ()) }
    });

    let context_merges = variant_names
        .iter()
        .zip(variant_types.iter())
        .map(|(_v_name, v_type)| {
            let search_type = common::replace_generic_with_search(v_type);
            quote! {
                let sub_ctx = <#search_type>::context(driver, options)?;
                ctx = ctx.merge(sub_ctx);
            }
        });

    let query_blocks = variant_names.iter().zip(variant_types.iter()).map(|(v_name, v_type)| {
        let search_type = common::replace_generic_with_search(v_type);
        quote! {
            let sub_query = <#search_type as ::svql_query::traits::Searchable>::instantiate(::svql_query::traits::Component::path(self).clone());
            let results = sub_query.query(driver, context, key, config);
            all_results.extend(results.into_iter().map(#enum_name::<::svql_query::Match>::#v_name));
        }
    });

    let expanded = quote! {
        #expanded_enum

        impl #impl_generics #enum_name #ty_generics #where_clause {
            #(#accessors)*
        }

        impl #impl_generics ::svql_query::traits::Component<S> for #enum_name #ty_generics #where_clause {
            fn path(&self) -> &::svql_query::instance::Instance {
                match self {
                    #(#component_arms_path),*,
                    Self::#abstract_ident { path, .. } => path,
                }
            }

            fn type_name(&self) -> &'static str {
                stringify!(#enum_name)
            }

            fn find_port(&self, path: &::svql_query::instance::Instance) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#component_arms_find_port),*,
                    Self::#abstract_ident { .. } => None,
                }
            }

            fn find_port_inner(&self, rel_path: &[std::sync::Arc<str>]) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#component_arms_find_port_inner),*,
                    Self::#abstract_ident { .. } => None,
                }
            }
        }

        impl ::svql_query::traits::Searchable for #enum_name<::svql_query::Search> {
            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self::#abstract_ident {
                    path: base_path.clone(),
                    #(#abstract_init_fields),*
                }
            }
        }

        impl #enum_name<::svql_query::Search> {

        }

        impl<'a> ::svql_query::traits::Reportable for #enum_name<::svql_query::Match> {
            fn to_report(&self, name: &str) -> ::svql_query::report::ReportNode {
                use ::svql_query::subgraph::cell::SourceLocation;

                match self {
                    #(
                        Self::#variant_names(inner) => {
                            let mut node = inner.to_report(name);
                            node.details = Some(stringify!(#variant_names).to_string());
                            node
                        }
                    )*,
                    _ => unreachable!("Abstract variant in report")
                }
            }
        }

        impl ::svql_query::traits::Query for #enum_name<::svql_query::Search> {
            type Matched<'a> = #enum_name<::svql_query::Match>;

            fn query<'a>(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &'a ::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Matched<'a>> {
                use ::svql_query::traits::Component;
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
