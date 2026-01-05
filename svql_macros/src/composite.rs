use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, parse_macro_input};

enum FieldKind {
    Path,
    Submodule,
    Wire,
}

struct CompositeField {
    ident: syn::Ident,
    ty: syn::Type,
    vis: syn::Visibility,
    kind: FieldKind,
}

pub fn composite_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    // --- Parsing Phase ---
    let mut fields_info = Vec::new();
    let mut path_ident = None;

    if let Fields::Named(ref fields) = item_struct.fields {
        for field in &fields.named {
            let ident = field.ident.clone().unwrap();
            let mut kind = FieldKind::Wire; // Default

            // Check attributes non-destructively
            for attr in &field.attrs {
                if attr.path().is_ident("path") {
                    kind = FieldKind::Path;
                } else if attr.path().is_ident("submodule") {
                    kind = FieldKind::Submodule;
                }
            }

            if let FieldKind::Path = kind {
                path_ident = Some(ident.clone());
            }

            fields_info.push(CompositeField {
                ident,
                ty: field.ty.clone(),
                vis: field.vis.clone(),
                kind,
            });
        }
    }

    let path_ident = path_ident.expect("Composite struct must have a #[path] field");

    // --- Generation Phase ---

    // 1. Struct Definition
    let struct_fields = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let vis = &f.vis;
        quote! { #vis #ident: #ty }
    });

    // 2. Instantiate (Search State)
    let instantiate_fields = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let name_str = ident.to_string();
        match f.kind {
            FieldKind::Path => quote! { #ident: base_path.clone() },
            FieldKind::Submodule => {
                let ty = &f.ty;
                quote! {
                    #ident: <<#ty as ::svql_query::traits::Projected>::Pattern as ::svql_query::traits::Searchable>::instantiate(base_path.child(#name_str))
                }
            },
            FieldKind::Wire => quote! {
                #ident: ::svql_query::Wire::new(base_path.child(#name_str), ())
            },
        }
    });

    // 3. Query Logic
    let mut query_calls = Vec::new();
    let mut query_vars = Vec::new();
    let mut construct_fields = Vec::new();

    for f in &fields_info {
        let ident = &f.ident;
        match f.kind {
            FieldKind::Submodule => {
                query_calls.push(quote! {
                    let #ident = self.#ident.query(driver, context, key, config);
                });
                query_vars.push(ident);
                construct_fields.push(quote! { #ident: #ident });
            }
            FieldKind::Path => {
                construct_fields.push(quote! { #ident: self.#ident.clone() });
            }
            FieldKind::Wire => {
                // Wires in composites are usually just placeholders or manual connections
                // In Match state, they remain empty/unbound unless manually bound later
                construct_fields.push(quote! {
                    #ident: ::svql_query::Wire::new(self.#ident.path.clone(), None)
                });
            }
        }
    }

    // 4. Context Building
    let context_calls = fields_info.iter().filter_map(|f| {
        if let FieldKind::Submodule = f.kind {
            let ty = &f.ty;
            // Use the robust generic replacement
            let search_ty = common::replace_state_generic(ty);
            Some(quote! {
                let sub_ctx = <#search_ty>::context(driver, options)?;
                ctx = ctx.merge(sub_ctx);
            })
        } else {
            None
        }
    });

    // 5. Find Port Logic
    let find_port_arms = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let name_str = ident.to_string();
        match f.kind {
            FieldKind::Path => quote! {}, // Path doesn't have ports
            _ => quote! {
                #name_str => self.#ident.find_port_inner(tail)
            },
        }
    });

    // 6. Reporting
    let report_children = fields_info.iter().filter_map(|f| {
        if let FieldKind::Submodule = f.kind {
            let ident = &f.ident;
            Some(quote! { self.#ident.to_report(stringify!(#ident)) })
        } else {
            None
        }
    });

    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #impl_generics #where_clause {
            #(#struct_fields),*
        }

        impl #impl_generics ::svql_query::traits::Projected for #struct_name<::svql_query::Search> #where_clause {
            type Pattern = #struct_name<::svql_query::Search>;
            type Result = #struct_name<::svql_query::Match>;
        }

        impl #impl_generics ::svql_query::traits::Projected for #struct_name<::svql_query::Match> #where_clause {
            type Pattern = #struct_name<::svql_query::Search>;
            type Result = #struct_name<::svql_query::Match>;
        }

        impl #impl_generics ::svql_query::traits::Component<S> for #struct_name #ty_generics #where_clause {
            fn path(&self) -> &::svql_query::instance::Instance {
                &self.#path_ident
            }

            fn type_name(&self) -> &'static str {
                stringify!(#struct_name)
            }

            fn find_port(&self, path: &::svql_query::instance::Instance) -> Option<&::svql_query::Wire<S>> {
                if !path.starts_with(self.path()) { return None; }
                let rel_path = path.relative(self.path());
                self.find_port_inner(rel_path)
            }

            fn find_port_inner(&self, rel_path: &[std::sync::Arc<str>]) -> Option<&::svql_query::Wire<S>> {
                let next = match rel_path.first() {
                    Some(arc_str) => arc_str.as_ref(),
                    None => return None,
                };
                let tail = &rel_path[1..];
                match next {
                    #(#find_port_arms),*,
                    _ => None,
                }
            }
        }

        impl ::svql_query::traits::Searchable for #struct_name<::svql_query::Search> {
            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self {
                    #(#instantiate_fields),*
                }
            }
        }

        impl<'a> ::svql_query::traits::Reportable for #struct_name<::svql_query::Match> {
            fn to_report(&self, name: &str) -> ::svql_query::report::ReportNode {
                let children = vec![
                    #(#report_children),*
                ];

                ::svql_query::report::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!(#struct_name).to_string(),
                    path: self.#path_ident.clone(),
                    details: None,
                    source_loc: None,
                    children,
                }
            }
        }

        impl ::svql_query::traits::Query for #struct_name<::svql_query::Search> {
            fn query<'a>(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &'a ::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Result> {
                use ::svql_query::prelude::{Component, Topology, ConnectionBuilder};
                ::svql_query::tracing::info!("{} searching composite", self.log_label());

                // 1. Execute sub-queries
                #(#query_calls)*

                // 2. Cartesian Product & Filtering
                // Note: We use ::svql_query::itertools::iproduct to ensure the macro works
                // without the user importing itertools explicitly.
                let results: Vec<_> = ::svql_query::itertools::iproduct!( #(#query_vars),* )
                    .map(|( #(#query_vars),* )| {
                        #struct_name {
                            #(#construct_fields),*
                        }
                    })
                    .filter(|candidate| {
                        let mut builder = ConnectionBuilder { constraints: Vec::new() };
                        candidate.define_connections(&mut builder);

                        let haystack_index = context.get(key).unwrap().index();

                        for group in builder.constraints {
                            let mut group_satisfied = false;
                            for (from_opt, to_opt) in group {
                                if let (Some(from), Some(to)) = (from_opt, to_opt) {
                                    if ::svql_query::traits::validate_connection(from, to, haystack_index) {
                                        group_satisfied = true;
                                        break;
                                    }
                                }
                            }
                            if !group_satisfied {
                                return false;
                            }
                        }
                        true
                    })
                    .collect();

                ::svql_query::tracing::info!("{} found {} matches", self.log_label(), results.len());
                results
            }

            fn context(
                driver: &::svql_query::driver::Driver,
                options: &::svql_query::common::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::driver::Context::new();
                #(#context_calls)*
                Ok(ctx)
            }
        }
    };

    TokenStream::from(expanded)
}
