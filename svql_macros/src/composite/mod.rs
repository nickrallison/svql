use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, parse_macro_input};

pub fn composite_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;
    let generics = &item_struct.generics; // Capture full generics with bounds
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let mut path_field = None;
    let mut submodules = Vec::new();
    let mut children_refs = Vec::new();
    let mut find_port_arms = Vec::new();
    let mut instantiate_fields = Vec::new();
    let mut query_calls = Vec::new();
    let mut query_names = Vec::new();
    let mut construct_fields = Vec::new();
    let mut field_defs = Vec::new();
    let mut context_calls = Vec::new();

    if let Fields::Named(ref mut fields) = item_struct.fields {
        for field in fields.named.iter_mut() {
            let ident = field.ident.as_ref().unwrap();
            let name_str = ident.to_string();
            let ty = &field.ty;
            let vis = &field.vis;

            // Check attributes
            let mut is_path = false;
            let mut is_submodule = false;
            let mut attrs_to_remove = Vec::new();

            for (i, attr) in field.attrs.iter().enumerate() {
                if attr.path().is_ident("path") {
                    is_path = true;
                    attrs_to_remove.push(i);
                } else if attr.path().is_ident("submodule") {
                    is_submodule = true;
                    attrs_to_remove.push(i);
                }
            }

            for i in attrs_to_remove.into_iter().rev() {
                field.attrs.remove(i);
            }

            field_defs.push(quote! { #vis #ident: #ty });

            if is_path {
                path_field = Some(ident.clone());
            } else if is_submodule {
                submodules.push((ident.clone(), ty.clone()));
                children_refs.push(quote! { &self.#ident });

                find_port_arms.push(quote! {
                    #name_str => self.#ident.find_port_inner(tail)
                });

                // Use helper to get Type<Search>
                let search_ty = common::replace_generic_with_search(ty);

                instantiate_fields.push(quote! {
	                    #ident: <#search_ty as ::svql_query::traits::Searchable>::instantiate(base_path.child(#name_str))
	                });

                query_calls.push(quote! {
                    let #ident = self.#ident.query(driver, context, key, config);
                });
                query_names.push(ident.clone());
                construct_fields.push(quote! { #ident: #ident });

                // Generate context call for this submodule
                // FIXED: Wrapped #search_ty in <...> to avoid "comparison operators cannot be chained"
                context_calls.push(quote! {
                    let sub_ctx = <#search_ty>::context(driver, options)?;
                    ctx = ctx.merge(sub_ctx);
                });
            } else {
                children_refs.push(quote! { &self.#ident });
                find_port_arms.push(quote! {
                    #name_str => self.#ident.find_port_inner(tail)
                });
                instantiate_fields.push(quote! {
                    #ident: ::svql_query::Wire::new(base_path.child(#name_str), ())
                });
            }
        }
    }

    let path_ident = path_field.expect("Composite struct must have a #[path] field");

    let iproduct_macro = quote! { ::svql_query::itertools::iproduct! };
    let iter_vars: Vec<_> = query_names.iter().collect();

    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #generics #where_clause {
            #(#field_defs),*
        }

        impl #impl_generics ::svql_query::traits::Component<S> for #struct_name #ty_generics #where_clause {
            fn path(&self) -> &::svql_query::instance::Instance {
                &self.#path_ident
            }

            fn type_name(&self) -> &'static str {
                stringify!(#struct_name)
            }

            fn children(&self) -> Vec<&dyn ::svql_query::traits::Component<S>> {
                vec![ #(#children_refs),* ]
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
                    #path_ident: base_path.clone(),
                    #(#instantiate_fields),*
                }
            }
        }

        impl #struct_name<::svql_query::Search> {
            pub fn context(
                driver: &::svql_query::svql_driver::Driver,
                options: &::svql_query::svql_common::ModuleConfig
            ) -> Result<::svql_query::svql_driver::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::svql_driver::Context::new();
                #(#context_calls)*
                Ok(ctx)
            }
        }

        impl ::svql_query::traits::Query for #struct_name<::svql_query::Search> {
            type Matched<'a> = #struct_name<::svql_query::Match<'a>>;

            fn query<'a>(
                &self,
                driver: &::svql_query::svql_driver::Driver,
                context: &'a ::svql_query::svql_driver::Context,
                key: &::svql_query::svql_driver::DriverKey,
                config: &::svql_query::svql_common::Config
            ) -> Vec<Self::Matched<'a>> {
                use ::svql_query::traits::Topology;

                // 1. Run Subqueries
                #(#query_calls)*

                // 2. Cartesian Product
                #iproduct_macro( #(#iter_vars),* )
                    .map(|( #(#iter_vars),* )| {
                        #struct_name {
                            #path_ident: self.#path_ident.clone(),
                            #(#construct_fields),*
                        }
                    })
                    .filter(|candidate| {
                        let mut builder = ::svql_query::traits::ConnectionBuilder { constraints: Vec::new() };
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
                    .collect()
            }
        }
    };

    TokenStream::from(expanded)
}
