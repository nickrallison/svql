use proc_macro2::TokenStream;
use quote::quote;

use super::lower::Ir;

pub fn codegen(ir: Ir) -> TokenStream {
    let name = &ir.name;
    let module_name = &ir.module_name;
    let file_path = &ir.file_path;

    // Field declarations and initializers for inputs and outputs
    let (input_decls, output_decls, input_inits, output_inits, input_binds, output_binds) = {
        let input_decls = ir.inputs.iter().map(|port| {
            let port_name = &port.name;
            quote! {
                pub #port_name: crate::Wire<S>,
            }
        });
        let output_decls = ir.outputs.iter().map(|port| {
            let port_name = &port.name;
            quote! {
                pub #port_name: crate::Wire<S>,
            }
        });
        let input_inits = ir.inputs.iter().map(|port| {
            let port_name = &port.name;
            let port_name_str = port_name.to_string();
            quote! {
                #port_name: crate::Wire::new(path.child(#port_name_str.to_string())),
            }
        });
        let output_inits = ir.outputs.iter().map(|port| {
            let port_name = &port.name;
            let port_name_str = port_name.to_string();
            quote! {
                #port_name: crate::Wire::new(path.child(#port_name_str.to_string())),
            }
        });
        let input_binds = ir.inputs.iter().map(|port| {
            let port_name = &port.name;
            let port_name_str = port_name.to_string();
            quote! {
                let #port_name = crate::binding::bind_input(
                    m,
                    #port_name_str,
                    0,
                    &embedding_set.needle_input_fanout_by_name,
                );
                let #port_name = crate::Wire::with_val(
                    path.child(#port_name_str.to_string()),
                    #port_name,
                );
            }
        });
        let output_binds = ir.outputs.iter().map(|port| {
            let port_name = &port.name;
            let port_name_str = port_name.to_string();
            quote! {
                let #port_name = crate::binding::bind_output(
                    m,
                    #port_name_str,
                    0,
                    &embedding_set.needle_output_fanin_by_name,
                );
                let #port_name = crate::Wire::with_val(
                    path.child(#port_name_str.to_string()),
                    #port_name,
                );
            }
        });
        (
            input_decls,
            output_decls,
            input_inits,
            output_inits,
            input_binds,
            output_binds,
        )
    };

    // Port specs for NetlistMeta
    let port_specs = ir.inputs.iter().chain(ir.outputs.iter()).map(|port| {
        let port_name_str = port.name.to_string();
        let dir = if ir.inputs.contains(&port) {
            quote! { crate::traits::netlist::PortDir::In }
        } else {
            quote! { crate::traits::netlist::PortDir::Out }
        };
        quote! {
            crate::traits::netlist::PortSpec {
                name: #port_name_str,
                dir: #dir,
            }
        }
    });

    // Match arms for find_port
    let find_port_arms = ir.inputs.iter().chain(ir.outputs.iter()).map(|port| {
        let port_name = &port.name;
        let port_name_str = port_name.to_string();
        quote! {
            Some(#port_name_str) => self.#port_name.find_port(p),
        }
    });

    // Field names for struct construction in from_subgraph
    let field_names = ir.inputs.iter().chain(ir.outputs.iter()).map(|port| {
        let port_name = &port.name;
        quote! { #port_name, }
    });

    quote! {
        #[derive(Debug, Clone)]
        pub struct #name<S>
        where
            S: crate::State,
        {
            pub path: crate::instance::Instance,
            #(#input_decls)*
            #(#output_decls)*
        }

        impl<S> #name<S>
        where
            S: crate::State,
        {
            pub fn new(path: crate::instance::Instance) -> Self {
                Self {
                    path: path.clone(),
                    #(#input_inits)*
                    #(#output_inits)*
                }
            }
        }

        impl<S> crate::WithPath<S> for #name<S>
        where
            S: crate::State,
        {
            fn find_port(
                &self,
                p: &crate::instance::Instance,
            ) -> Option<&crate::Wire<S>> {
                let idx = self.path.height() + 1;
                match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
                    #(#find_port_arms)*
                    _ => None,
                }
            }
            fn path(&self) -> crate::instance::Instance {
                self.path.clone()
            }
        }

        impl crate::traits::netlist::NetlistMeta for #name<crate::Search> {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;
            const PORTS: &'static [crate::traits::netlist::PortSpec] = &[
                #(#port_specs),*
            ];
        }

        impl crate::traits::netlist::SearchableNetlist for #name<crate::Search> {
            type Hit<'ctx> = #name<crate::Match<'ctx>>;
            fn from_subgraph<'ctx>(
                m: &svql_subgraph::Embedding<'ctx, 'ctx>,
                path: crate::instance::Instance,
                embedding_set: &svql_subgraph::EmbeddingSet<'ctx, 'ctx>,
            ) -> Self::Hit<'ctx> {
                #(#input_binds)*
                #(#output_binds)*
                Self::Hit::<'ctx> {
                    path: path.clone(),
                    #(#field_names)*
                }
            }
        }
    }
}
