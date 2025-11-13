use proc_macro2::TokenStream;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let fields = &ir.subs;
    let connections = &ir.connections; // Now Vec<Vec<ConnectionRef>>

    // Generate field declarations
    let field_decls = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let ty = &sub.ty;
        quote! {
            pub #field_name: #ty<S>
        }
    });

    // FIXED: Generate field initializers in new() – use instance method path.child(...) (not static Instance::child)
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
    let parallel_let_binding_fields = fields.iter().map(|sub| &sub.field_name);
    let sequential_let_binding_fields = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_iproduct = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_map_pattern = fields.iter().map(|sub| &sub.field_name);
    let field_names_for_struct_construction = fields.iter().map(|sub| &sub.field_name);

    // Generate connection groups: vec![ vec![conns...], vec![conns...] ]
    let connection_groups = connections.iter().map(|group| {
        let group_items = group.iter().map(|conn| {
            let from_sub = &conn.from_sub;
            let from_port = &conn.from_port;
            let to_sub = &conn.to_sub;
            let to_port = &conn.to_port;
            quote! {
                crate::Connection {
                    from: self.#from_sub.#from_port.clone(),
                    to: self.#to_sub.#to_port.clone(),
                }
            }
        });
        quote! {
            vec![ #(#group_items),* ]
        }
    });

    // Generate context merging
    let context_calls = fields.iter().map(|sub| {
        let ty = &sub.ty;
        quote! {
            <#ty<crate::Search>>::context(driver, config)?
        }
    });

    // Generate parallel query spawns
    let parallel_spawns = fields.iter().map(|sub| {
        let field_name = &sub.field_name;
        let ty = &sub.ty;
        let field_name_str = field_name.to_string();
        quote! {
            let #field_name = scope.spawn(|| {
                <#ty<crate::Search>>::query(
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
            <#ty<crate::Search>>::query(
                haystack_key,
                context,
                path.child(#field_name_str.to_string()),
                config,
            )
        }
    });

    let query_log_msg = format!("{}::query: executing with parallel queries", name);
    let query_log_msg_seq = format!("{}::query: executing sequential queries", name);

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct #name<S>
        where
            S: crate::State,
        {
            pub path: crate::instance::Instance,
            #(#field_decls,)*
        }

        impl<S> #name<S>
        where
            S: crate::State,
        {
            pub fn new(path: crate::instance::Instance) -> Self {
                Self {
                    path: path.clone(),
                    #(#field_inits,)*
                }
            }
        }

        impl<S> crate::WithPath<S> for #name<S>
        where
            S: crate::State,
        {
            crate::impl_find_port!(#name, #(#field_names_for_find_port),*);

            fn path(&self) -> crate::instance::Instance {
                self.path.clone()
            }
        }

        impl<S> crate::traits::composite::Composite<S> for #name<S>
        where
            S: crate::State,
        {
            fn connections(&self) -> Vec<Vec<crate::Connection<S>>> {
                vec![
                    #(#connection_groups),*
                ]
            }
        }

        impl<'ctx> crate::traits::composite::MatchedComposite<'ctx> for #name<crate::Match<'ctx>> {}

        impl crate::traits::composite::SearchableComposite for #name<crate::Search> {
            type Hit<'ctx> = #name<crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::Context, Box<dyn std::error::Error>> {
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
                haystack_key: &svql_driver::DriverKey,
                context: &'ctx svql_driver::Context,
                path: crate::instance::Instance,
                config: &svql_common::Config,
            ) -> Vec<Self::Hit<'ctx>> {

                let haystack_index = context.get(haystack_key).unwrap().index();

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

                itertools::iproduct!(#(#field_names_for_iproduct),*)
                    .map(|(#(#field_names_for_map_pattern),*)| #name {
                        path: path.clone(),
                        #(#field_names_for_struct_construction),*
                    })
                    .filter(|composite| composite.validate_connections(composite.connections(), haystack_index))
                    .collect()
            }
        }
    }
}
