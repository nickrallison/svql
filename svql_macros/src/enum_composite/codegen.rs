use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let variants = &ir.variants;

    if variants.is_empty() {
        return quote! {
            #[derive(Debug, Clone)]
            pub enum #name<S> where S: crate::State { }
        };
    }

    let enum_variants = variants.iter().map(|v| {
        let variant_name = &v.variant_name;
        let ty = &v.ty;
        quote! { #variant_name(#ty<S>) }
    });

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

    let context_calls = variants.iter().map(|v| {
        let ty = &v.ty;
        quote! {
            <#ty<crate::Search> as crate::traits::netlist::SearchableNetlist>::context(driver, config)?
        }
    });

    let parallel_spawns = variants.iter().map(|v| {
        let var_name = &v.var_name;
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        quote! {
            let #var_name = scope.spawn(|| {
                <#ty<crate::Search> as crate::traits::netlist::SearchableNetlist>::query(
                    haystack_key, context, path.child(#inst_name.to_string()), config
                )
            });
        }
    });

    let parallel_let_binding_fields = variants.iter().map(|v| &v.var_name);
    let parallel_joins = variants.iter().map(|v| {
        let var_name = &v.var_name;
        let error_msg = format!("Failed to join {} thread", var_name);
        quote! { #var_name.join().expect(#error_msg) }
    });

    let sequential_let_binding_fields = variants.iter().map(|v| &v.var_name);
    let sequential_queries = variants.iter().map(|v| {
        let ty = &v.ty;
        let inst_name = &v.inst_name;
        quote! {
            <#ty<crate::Search> as crate::traits::netlist::SearchableNetlist>::query(
                haystack_key, context, path.child(#inst_name.to_string()), config
            )
        }
    });

    let parallel_extend_arms = variants.iter().map(|v| {
        let var_name = &v.var_name;
        let variant_name = &v.variant_name;
        let bound = format_ident!("hit");
        quote! {
            all_hits.extend(#var_name.into_iter().map(|#bound| #name::<crate::Match<'ctx>>::#variant_name(#bound)));
        }
    });

    let sequential_extend_arms = variants.iter().map(|v| {
        let var_name = &v.var_name;
        let variant_name = &v.variant_name;
        let bound = format_ident!("hit");
        quote! {
            all_hits.extend(#var_name.into_iter().map(|#bound| #name::<crate::Match<'ctx>>::#variant_name(#bound)));
        }
    });

    let query_log_msg = format!("{}::query: executing with parallel queries", name);
    let query_log_msg_seq = format!("{}::query: executing sequential queries", name);

    // NEW: Generate common port accessors
    let common_port_accessors = ir.common_ports.iter().map(|port| {
        let method_name = &port.method_name;
        let field_name = &port.field_name;
        let field_name_str = field_name.to_string();

        let match_arms = variants.iter().map(|v| {
            let variant_name = &v.variant_name;
            let bound = format_ident!("inner");
            quote! { #name::#variant_name(#bound) => &#bound.#field_name }
        });

        quote! {
            #[doc = concat!("Access common port `", #field_name_str, "` shared by all variants.")]
            pub fn #method_name(&self) -> &crate::Wire<S> {
                match self {
                    #(#match_arms),*
                }
            }
        }
    });

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

        impl<S> crate::traits::enum_composite::EnumComposite<S> for #name<S>
        where
            S: crate::State,
        {
        }

        impl<'ctx> crate::traits::enum_composite::MatchedEnumComposite<'ctx> for #name<crate::Match<'ctx>> {
        }

        impl crate::traits::enum_composite::SearchableEnumComposite for #name<crate::Search> {
            type Hit<'ctx> = #name<crate::Match<'ctx>>;

            fn context(
                driver: &svql_driver::Driver,
                config: &svql_common::ModuleConfig,
            ) -> Result<svql_driver::Context, Box<dyn std::error::Error>> {
                let contexts = vec![ #(#context_calls,)* ];
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

        // NEW: Common port accessor methods
        impl<S> #name<S>
        where
            S: crate::State,
        {
            #(#common_port_accessors)*
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    fn codegen_compiles(ts: proc_macro2::TokenStream) -> bool {
        parse2::<syn::File>(ts).is_ok()
    }

    #[test]
    fn common_ports_codegen_valid_syntax() {
        let input = quote! {
            name: TestEnum,
            variants: [
                (A, "a", TypeA),
                (B, "b", TypeB)
            ],
            common_ports: {
                x: "get_x",
                y: "get_y"
            }
        };

        let ast = super::super::parse::parse(input);
        let model = super::super::analyze::analyze(ast);
        let ir = super::super::lower::lower(model);
        let code = super::super::codegen::codegen(ir);

        assert!(
            codegen_compiles(code.clone()),
            "Generated code should be valid Rust:\n{}",
            code
        );
    }

    #[test]
    fn common_ports_generates_methods() {
        let input = quote! {
            name: DffTest,
            variants: [(Simple, "s", SimpleDff)],
            common_ports: { clk: "clock" }
        };

        let ast = super::super::parse::parse(input);
        let model = super::super::analyze::analyze(ast);
        let ir = super::super::lower::lower(model);
        let code = super::super::codegen::codegen(ir);

        let code_str = code.to_string();
        assert!(
            code_str.contains("pub fn clock"),
            "Should generate clock() method"
        );
        assert!(
            code_str.contains("Access common port"),
            "Should have doc comment"
        );
    }

    #[test]
    fn empty_common_ports_no_methods() {
        let input = quote! {
            name: NoCommon,
            variants: [(A, "a", T)]
        };

        let ast = super::super::parse::parse(input);
        let model = super::super::analyze::analyze(ast);
        let ir = super::super::lower::lower(model);
        let code = super::super::codegen::codegen(ir);

        let code_str = code.to_string();
        assert!(code_str.contains("impl < S >"), "Should have impl block");
    }
}
