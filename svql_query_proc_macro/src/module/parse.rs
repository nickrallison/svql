use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use syn::{parse2, AttributeArgs, Ident, ItemStruct, Lit, Meta, NestedMeta, Visibility};

pub struct Ast {
    pub vis: Visibility,
    pub iface_ident: Ident,
    pub file_path: String,
    pub module_name: String,
    pub yosys: String,
    pub svql_pat_plugin_path: String,
}

pub fn parse(attr: TokenStream, item: TokenStream) -> Ast {
    let attr_args: AttributeArgs = parse2(attr).expect_or_abort("cannot parse attribute arguments");

    let mut file_path: Option<String> = None;
    let mut module_name: Option<String> = None;
    let mut yosys: Option<String> = None;
    let mut svql_pat_plugin_path: Option<String> = None;

    for arg in attr_args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("file") => match nv.value {
                Lit::Str(s) => file_path = Some(s.value()),
                _ => abort!(nv.value, "`file` must be a string literal"),
            },
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("module") => match nv.value {
                Lit::Str(s) => module_name = Some(s.value()),
                _ => abort!(nv.value, "`module` must be a string literal"),
            },
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("yosys") => match nv.value {
                Lit::Str(s) => yosys = Some(s.value()),
                _ => abort!(nv.value, "`yosys` must be a string literal"),
            },
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("svql_pat_plugin_path") => {
                match nv.value {
                    Lit::Str(s) => svql_pat_plugin_path = Some(s.value()),
                    _ => abort!(nv.value, "`svql_pat_plugin_path` must be a string literal"),
                }
            }
            other => abort!(other, "unsupported attribute argument"),
        }
    }

    let file_path = file_path.expect("`file = \"...\"` is required");
    let module_name = module_name.expect("`module = \"...\"` is required");
    let yosys = yosys.unwrap_or_else(|| "yosys".to_string());
    let svql_pat_plugin_path =
        svql_pat_plugin_path.expect("`svql_pat_plugin_path = \"...\"` is required");

    let item_struct: ItemStruct =
        parse2(item).expect_or_abort("#[module] must be used on a struct");

    if !item_struct.generics.params.is_empty() {
        abort!(
            item_struct.ident,
            "#[module] does not support generic structs"
        );
    }

    Ast {
        vis: item_struct.vis,
        iface_ident: item_struct.ident,
        file_path,
        module_name,
        yosys,
        svql_pat_plugin_path,
    }
}
