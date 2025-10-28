use proc_macro2::TokenStream;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let variants = &ir.variants;

    if variants.is_empty() {
        // Handle empty case (though unlikely)
        return quote! {
            #[derive(Debug, Clone)]
            pub enum #name<S> where S: crate::State {}
            // Minimal impls...
        };
    }

    // Generate enum variants: VariantName(Type<S>)
    let enum_variants = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.ty;
        quote! {
            #variant_name(#ty<S>)
        }
    });

    // Generate Debug impl (match arms)
    let debug_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let inner = &v.variant_name;
        quote! {
            #name::#variant_name(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, stringify!(#variant_name), &__self_0)
            }
        }
    });

    // Generate Clone impl (match arms)
    let clone_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let inner = &v.variant_name;
        quote! {
            #name::#variant_name(__self_0) => #name::#variant_name(::core::clone::Clone::clone(__self_0))
        }
    });

    // Generate WithPath match arms (delegate to inner)
    let withpath_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let inner = &v.variant_name;
        quote! {
            #name::#variant_name(#inner) => #inner.find_port(p),
        }
    });
    let withpath_path_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let inner = &v.variant_name;
        quote! {
            #name::#variant_name(#inner) => #inner.path(),
        }
    });

    // Generate context merging
    let context_calls = variants.iter().map(|v| {
        let ty = &v.ty;
        quote! {
            <#ty<crate::Search>>::context(driver, config)?
        }
    });

    // Generate parallel spawns and joins
    let parallel_spawns = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        quote! {
            let #variant_name = scope.spawn(|| {
                <#ty<crate::Search>>::query(
                    haystack_key,
                    context,
                    path.child(#inst_name.to_string()),
                    config,
                )
            });
        }
    });
    let parallel_joins = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let error_msg = format!("Failed to join {} thread", variant_name);
        quote! {
            #variant_name.join().expect(#error_msg)
        }
    });

    // Generate sequential queries
    let sequential_queries = variants.iter().map(|v| {
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        let variant_name = &v.variant_name;
        quote! {
            let #variant_name = <#ty<crate::Search>>::query(
                haystack_key,
                context,
                path.child(#inst_name.to_string()),
                config,
            );
        }
    });

    // Generate mapping to enum variants and chaining
    let chain_mappings = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.variant_name; // Use variant name for mapping
        quote! {
            #variant_name.into_iter().map(#name::<crate::Match<'ctx>>::#ty)
        }
    });

    let query_log_msg = format!("{}::query: executing with parallel queries", name);
    let query_log_msg_seq = format!("{}::query: executing sequential queries", name);

    quote! {
        #[derive(Debug, Clone)]
        pub enum #name<S>
        where
            S: crate::State,
        {
            #(#enum_variants,)*
        }

        #[automatically_derived]
        impl<S: ::core::fmt::Debug> ::core::fmt::Debug for #name<S>
        where
            S: crate::State,
        {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    #(#debug_arms)*
                }
            }
        }

        #[automatically_derived]
        impl<S: ::core::clone::Clone> ::core::clone::Clone for #name<S>
        where
            S: crate::State,
        {
            #[inline]
            fn clone(&self) -> #name<S> {
                match self {
                    #(#clone_arms)*
                }
            }
        }

        impl<S> crate::WithPath<S> for #name<S>
        where
            S: crate::State,
        {
            fn find_port(&self, p: &crate::instance::Instance) -> Option<&crate::Wire<S>> {
                match self {
                    #(#withpath_arms)*
                }
            }

            fn path(&self) -> crate::instance::Instance {
                match self {
                    #(#withpath_path_arms)*
                }
            }
        }

        impl<S> crate::composite::EnumComposite<S> for #name<S>
        where
            S: crate::State,
        {}

        impl<'ctx> crate::composite::MatchedEnumComposite<'ctx> for #name<crate::Match<'ctx>> {}

        impl crate::composite::SearchableEnumComposite for #name<crate::Search> {
            type Hit<'ctx> = #name<crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::Context, Box<dyn std::error::Error>> {
                let contexts = vec![
                    #(#context_calls,)*
                ];
                let mut iter = contexts.into_iter();
                let mut result = iter.next().ok_or("No variants defined")?;
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
                #[cfg(feature = "parallel")]
                let (#(#parallel_joins,)*) = {
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
                {
                    ::tracing::event!(
                        ::tracing::Level::INFO,
                        #query_log_msg_seq
                    );

                    #(#sequential_queries)*
                };

                (#(#parallel_joins,)*)  // For parallel case
                    #(#sequential_queries)*  // For sequential case (already bound)

                #(#chain_mappings)*
                    .collect()
            }
        }
    }
}
