// svql_macros/src/enum_composite/codegen.rs
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let variants = &ir.variants;

    if variants.is_empty() {
        // Handle empty case (though unlikely)
        return quote! {
            #[derive(Debug, Clone)]
            pub enum #name<S>
            where
                S: crate::State,
            {
            }

            impl<S> crate::WithPath<S> for #name<S>
            where
                S: crate::State,
            {
                fn find_port(&self, _p: &crate::instance::Instance) -> Option<&crate::Wire<S>> {
                    None
                }

                fn path(&self) -> crate::instance::Instance {
                    crate::instance::Instance::root("".to_string())
                }
            }

            impl<S> crate::composite::EnumComposite<S> for #name<S>
            where
                S: crate::State,
            {
            }

            impl<'ctx> crate::composite::MatchedEnumComposite<'ctx> for #name<crate::Match<'ctx>> {
            }

            impl crate::composite::SearchableEnumComposite for #name<crate::Search> {
                type Hit<'ctx> = #name<crate::Match<'ctx>>;

                fn context(
                    _driver: &svql_driver::Driver,
                    _config: &svql_common::ModuleConfig,
                ) -> Result<svql_driver::Context, Box<dyn std::error::Error>> {
                    Ok(svql_driver::Context::default())
                }

                fn query<'ctx>(
                    _haystack_key: &svql_driver::DriverKey,
                    _context: &'ctx svql_driver::Context,
                    _path: crate::instance::Instance,
                    _config: &svql_common::Config,
                ) -> Vec<Self::Hit<'ctx>> {
                    vec![]
                }
            }
        };
    }

    // Generate enum variants: VariantName(Type<S>)
    let enum_variants = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.ty;
        quote! { #variant_name(#ty<S>) }
    });

    // Generate Debug impl (match arms)
    let debug_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        quote! {
            #name::#variant_name(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, stringify!(#variant_name), &__self_0)
            }
        }
    });

    // Generate Clone impl (match arms)
    let clone_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        quote! {
            #name::#variant_name(__self_0) => #name::#variant_name(::core::clone::Clone::clone(__self_0))
        }
    });

    // Generate WithPath match arms (delegate to inner) - FIXED: Use fresh bound ident
    let withpath_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let bound = format_ident!("inner");
        quote! { #name::#variant_name(#bound) => #bound.find_port(p) }
    });
    let withpath_path_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let bound = format_ident!("inner");
        quote! { #name::#variant_name(#bound) => #bound.path() }
    });

    // Generate context merging (one per variant) - FIXED: Use fully qualified trait syntax
    let context_calls = variants.iter().map(|v| {
        let ty = &v.ty;
        quote! { <#ty<crate::Search> as crate::netlist::SearchableNetlist>::context(driver, config)? }
    });

    // Parallel: Spawns, joins, and binding patterns - FIXED: Use fully qualified trait syntax
    let parallel_spawns = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        quote! {
            let #variant_name = scope.spawn(|| {
                <#ty<crate::Search> as crate::netlist::SearchableNetlist>::query(
                    haystack_key,
                    context,
                    path.child(#inst_name.to_string()),
                    config,
                )
            });
        }
    });

    let parallel_let_binding_fields = variants.iter().map(|v| &v.variant_name);
    let parallel_joins = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let error_msg = format!("Failed to join {} thread", variant_name);
        quote! { #variant_name.join().expect(#error_msg) }
    });

    // Sequential: Queries - FIXED: Use fully qualified trait syntax
    let sequential_let_binding_fields = variants.iter().map(|v| &v.variant_name);
    let sequential_queries = variants.iter().map(|v| {
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        quote! {
            <#ty<crate::Search> as crate::netlist::SearchableNetlist>::query(
                haystack_key,
                context,
                path.child(#inst_name.to_string()),
                config,
            )
        }
    });

    // FIXED: Inline the extend arms directly in each cfg block to avoid move issues
    let parallel_extend_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let bound = format_ident!("hit");
        quote! {
            all_hits.extend(#variant_name.into_iter().map(|#bound| #name::<crate::Match<'ctx>>::#variant_name(#bound)));
        }
    });

    let sequential_extend_arms = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let bound = format_ident!("hit");
        quote! {
            all_hits.extend(#variant_name.into_iter().map(|#bound| #name::<crate::Match<'ctx>>::#variant_name(#bound)));
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
            #(#enum_variants),*
        }

        impl<S> crate::WithPath<S> for #name<S>
        where
            S: crate::State,
        {
            fn find_port(&self, p: &crate::instance::Instance) -> Option<&crate::Wire<S>> {
                match self {
                    #(#withpath_arms),*
                }
            }

            fn path(&self) -> crate::instance::Instance {
                match self {
                    #(#withpath_path_arms),*
                }
            }
        }

        impl<S> crate::composite::EnumComposite<S> for #name<S>
        where
            S: crate::State,
        {
        }

        impl<'ctx> crate::composite::MatchedEnumComposite<'ctx> for #name<crate::Match<'ctx>> {
        }

        impl crate::composite::SearchableEnumComposite for #name<crate::Search> {
            type Hit<'ctx> = #name<crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::Context, Box<dyn std::error::Error>> {
                let contexts = vec![ #(#context_calls,)* ];
                let mut iter = contexts.into_iter();
                let mut result = iter.next().ok_or("No variants defined")?;
                for ctx in iter {
                    result = result.merge(ctx)?;
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
                {
                    ::tracing::event!(::tracing::Level::INFO, #query_log_msg);

                    let (#(#parallel_let_binding_fields),*) = ::std::thread::scope(|scope| {
                        #(#parallel_spawns)*
                        ( #(#parallel_joins,)* )
                    });

                    let mut all_hits: Vec<Self::Hit<'ctx>> = vec![];
                    #(#parallel_extend_arms)*
                    all_hits
                }

                #[cfg(not(feature = "parallel"))]
                {
                    ::tracing::event!(::tracing::Level::INFO, #query_log_msg_seq);

                    let (#(#sequential_let_binding_fields),*) = ( #(#sequential_queries,)* );

                    let mut all_hits: Vec<Self::Hit<'ctx>> = vec![];
                    #(#sequential_extend_arms)*
                    all_hits
                }
            }
        }
    }
}
