use proc_macro2::TokenStream;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let fields = &ir.subs;
    let connections = &ir.connections;

    // Generate field declarations
    let field_decls = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let ty = &sub.ty;
        quote! {
            pub #field_name: #ty<S>
        }
    });

    // Generate field initializers in new()
    let field_inits = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let ty = &sub.ty;
        let field_name_str = field_name.to_string();
        quote! {
            #field_name: <#ty<S>>::new(path.child(#field_name_str.to_string()))
        }
    });

    // Field names for various uses
    let field_names_for_find_port = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_let_binding = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_iproduct = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_map_pattern = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_struct_construction = fields.iter().map(|sub| &sub.field_name);

    // Generate connections
    let connection_items = connections.iter().map(|conn| {
        let from_sub = &conn.from_sub;
        let from_port = &conn.from_port;
        let to_sub = &conn.to_sub;
        let to_port = &conn.to_port;
        quote! {
            ::svql_query::Connection {
                from: self.#from_sub.#from_port.clone(),
                to: self.#to_sub.#to_port.clone(),
            }
        }
    });

    // Generate context merging
    let context_calls = fields.iter().map(|sub| {
        let ty = &sub.ty;
        quote! {
            <#ty<::svql_query::Search>>::context(driver, config)?
        }
    });

    // Generate parallel query spawns
    let parallel_spawns = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let ty = &sub.ty;
        let field_name_str = field_name.to_string();
        quote! {
            let #field_name = scope.spawn(|| {
                <#ty<::svql_query::Search>>::query(
                    haystack_key,
                    context,
                    path.child(#field_name_str.to_string()),
                    config,
                )
            });
        }
    });

    let parallel_joins = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let error_msg = format!("Failed to join {} thread", field_name);
        quote! {
            #field_name.join().expect(#error_msg)
        }
    });

    // Generate sequential queries
    let sequential_queries = fields.iter().map(|sub| {
        let ty = &sub.ty;
        let field_name_str = sub.field_name.to_string();
        quote! {
            <#ty<::svql_query::Search>>::query(
                haystack_key,
                context,
                path.child(#field_name_str.to_string()),
                config,
            )
        }
    });

    let query_log_msg = format!("{}::query: executing with parallel queries", name);
    let query_log_msg_seq = format!("{}::query: executing sequential queries", name);

    let parallel_let_binding_fields = fields.iter().map(|sub| &sub.field_name);
    let sequential_let_binding_fields = fields.iter().map(|sub| &sub.field_name);

    quote! {
        #[derive(Debug, Clone)]
        pub struct #name<S>
        where
            S: ::svql_query::State,
        {
            pub path: ::svql_query::instance::Instance,
            #(#field_decls,)*
        }

        impl<S> #name<S>
        where
            S: ::svql_query::State,
        {
            pub fn new(path: ::svql_query::instance::Instance) -> Self {
                Self {
                    path: path.clone(),
                    #(#field_inits,)*
                }
            }
        }

        impl<S> ::svql_query::WithPath<S> for #name<S>
        where
            S: ::svql_query::State,
        {
            ::svql_query::impl_find_port!(#name, #(#field_names_for_find_port),*);

            fn path(&self) -> ::svql_query::instance::Instance {
                self.path.clone()
            }
        }

        impl<S> ::svql_query::composite::Composite<S> for #name<S>
        where
            S: ::svql_query::State,
        {
            fn connections(&self) -> Vec<Vec<::svql_query::Connection<S>>> {
                vec![vec![
                    #(#connection_items,)*
                ]]
            }
        }

        impl<'ctx> ::svql_query::composite::MatchedComposite<'ctx> for #name<::svql_query::Match<'ctx>> {}

        impl ::svql_query::composite::SearchableComposite for #name<::svql_query::Search> {
            type Hit<'ctx> = #name<::svql_query::Match<'ctx>>;

            fn context(
                driver: &::svql_driver::Driver,
                config: &::svql_common::ModuleConfig,
            ) -> Result<::svql_driver::Context, Box<dyn std::error::Error>> {
                let contexts = vec![
                    #(#context_calls,)*
                ];

                let mut iter = contexts.into_iter();
                let mut result = iter.next().ok_or("No sub-patterns defined")?;
                for ctx in iter {
                    result = result.merge(ctx);
                }
                Ok(result)
            }

            fn query<'ctx>(
                haystack_key: &::svql_driver::DriverKey,
                context: &'ctx ::svql_driver::Context,
                path: ::svql_query::instance::Instance,
                config: &::svql_common::Config,
            ) -> Vec<Self::Hit<'ctx>> {
                #[cfg(feature = "parallel")]
                let (#(#parallel_let_binding_fields),*) = {
                    ::tracing::event!(
                        ::tracing::Level::INFO,
                        #query_log_msg
                    );

                    ::std::thread::scope(|scope| {
                        #(#parallel_spawns)*

                        (
                            #(#parallel_joins,)*
                        )
                    })
                };

                #[cfg(not(feature = "parallel"))]
                let (#(#sequential_let_binding_fields),*) = {
                    ::tracing::event!(
                        ::tracing::Level::INFO,
                        #query_log_msg_seq
                    );

                    (
                        #(#sequential_queries,)*
                    )
                };

                ::itertools::iproduct!(#(#field_names_for_iproduct),*)
                    .map(|(#(#field_names_for_map_pattern),*)| #name {
                        path: path.clone(),
                        #(#field_names_for_struct_construction,)*
                    })
                    .filter(|composite| composite.validate_connections(composite.connections()))
                    .collect()
            }
        }
    }
}
