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

    // Generics
    let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();
    let generics = &item_enum.generics;

    // Specialized generics (S removed)
    let specialized_generics = common::remove_state_generic(&item_enum.generics);
    let (spec_impl_generics, _, spec_where_clause) = specialized_generics.split_for_impl();

    // Concrete types
    let search_type =
        common::make_replaced_type(enum_name, &item_enum.generics, quote!(::svql_query::Search));
    let match_type =
        common::make_replaced_type(enum_name, &item_enum.generics, quote!(::svql_query::Match));

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
    let port_names: Vec<String> = common_ports.iter().map(|p| p.to_string()).collect();

    // 1. Enum Definition
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

    // 3. Hardware Implementation
    let path_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! { Self::#ident(inner) => inner.path() }
    });

    let children_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! { Self::#ident(inner) => vec![inner] }
    });

    let find_port_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;

        let port_checks = common_ports.iter().map(|port| {
            let port_str = port.to_string();
            if let Some(Some(mapped_name)) = v.port_map.get(&port_str) {
                quote! {
                    if first_segment.as_ref() == #port_str {
                        let target_path = self.path().child(#mapped_name);
                        return inner.find_port(&target_path);
                    }
                }
            } else {
                quote! {}
            }
        });

        quote! {
            Self::#ident(inner) => {
                if !path.starts_with(self.path()) { return None; }
                let rel = path.relative(self.path());
                if let Some(first_segment) = rel.first() {
                    #(#port_checks)*
                }
                inner.find_port(path)
            }
        }
    });

    let report_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        quote! {
            Self::#ident(inner) => {
                let mut node = inner.report(name);
                node.details = Some(stringify!(#ident).to_string());
                node
            }
        }
    });

    // Variant name arms for VariantMatched
    let variant_name_arms = variants_info.iter().map(|v| {
        let ident = &v.ident;
        let name_str = ident.to_string();
        quote! { Self::#ident(_) => #name_str }
    });

    // 4. SearchableComponent & VariantComponent Implementation
    let abstract_init = common_ports.iter().map(|p| {
        let p_str = p.to_string();
        quote! { #p: ::svql_query::Wire::new(base_path.child(#p_str), ()) }
    });

    let query_blocks = variants_info.iter().map(|v| {
        let v_ident = &v.ident;
        let v_ty = &v.ty;
        let search_type = common::replace_state_generic(v_ty);

        quote! {
            {
                let sub_query = <#search_type as ::svql_query::traits::SearchableComponent>::create_at(
                    ::svql_query::traits::Hardware::path(self).clone()
                );
                let results = <#search_type as ::svql_query::traits::SearchableComponent>::execute_search(
                    &sub_query,
                    driver,
                    context,
                    key,
                    config
                );

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
            let sub_ctx = <#search_type as ::svql_query::traits::SearchableComponent>::build_context(driver, options)?;
            ctx = ctx.merge(sub_ctx);
        }
    });

    let expanded = quote! {
        #expanded_enum

        impl #impl_generics #enum_name #ty_generics #where_clause {
            #(#accessors)*
        }

        // Hardware implementation (state-generic)
        impl #impl_generics ::svql_query::traits::Hardware for #enum_name #ty_generics #where_clause {
            type State = S;

            fn path(&self) -> &::svql_query::instance::Instance {
                match self {
                    #(#path_arms),*,
                    Self::#abstract_ident { path, .. } => path,
                }
            }

            fn type_name(&self) -> &'static str {
                stringify!(#enum_name)
            }

            fn children(&self) -> Vec<&dyn ::svql_query::traits::Hardware<State = Self::State>> {
                match self {
                    #(#children_arms),*,
                    Self::#abstract_ident { .. } => vec![],
                }
            }

            fn find_port(&self, path: &::svql_query::instance::Instance) -> Option<&::svql_query::Wire<S>> {
                match self {
                    #(#find_port_arms),*,
                    Self::#abstract_ident { .. } => None,
                }
            }

            fn report(&self, name: &str) -> ::svql_query::report::ReportNode {
                match self {
                    #(#report_arms),*,
                    Self::#abstract_ident { .. } => panic!("__Abstract variant found in Match state during reporting."),
                }
            }
        }

        // SearchableComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::SearchableComponent for #search_type #spec_where_clause {
            type Kind = ::svql_query::traits::kind::Variant;
            type Match = #match_type;

            fn create_at(base_path: ::svql_query::instance::Instance) -> Self {
                Self::#abstract_ident {
                    path: base_path.clone(),
                    #(#abstract_init),*
                }
            }

            fn build_context(
                driver: &::svql_query::driver::Driver,
                options: &::svql_query::common::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::driver::Context::new();
                #(#context_merges)*
                Ok(ctx)
            }

            fn execute_search(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::traits::VariantComponent;
                self.search_variants(driver, context, key, config)
            }
        }

        // VariantComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::VariantComponent for #search_type #spec_where_clause {
            const COMMON_PORTS: &'static [&'static str] = &[#(#port_names),*];

            fn search_variants(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Match> {
                ::svql_query::tracing::info!("{} searching variants", ::svql_query::traits::Hardware::type_name(self));

                let mut all_results = Vec::new();
                #(#query_blocks)*

                ::svql_query::tracing::info!("{} found {} total matches across variants", ::svql_query::traits::Hardware::type_name(self), all_results.len());
                all_results
            }
        }

        // MatchedComponent implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::MatchedComponent for #match_type #spec_where_clause {
            type Search = #search_type;
        }

        // VariantMatched implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::VariantMatched for #match_type #spec_where_clause {
            type SearchType = #search_type;

            fn variant_name(&self) -> &'static str {
                match self {
                    #(#variant_name_arms),*,
                    Self::#abstract_ident { .. } => panic!("__Abstract variant in Match state"),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
