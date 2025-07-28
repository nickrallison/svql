use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse2, Ident, ItemStruct, Lit, Meta, NestedMeta, Visibility};

pub struct Ast {
    pub vis: Visibility,
    pub iface_ident: Ident,
    pub file_path: String,
    pub module_name: String,
    pub yosys: String,
    pub svql_pat_plugin_path: String,
}

pub fn parse(attr: TokenStream, item: TokenStream) -> Ast {
    // 1) parse the comma-separated list of NestedMeta
    let raw_args = Punctuated::<NestedMeta, Comma>::parse_terminated
        .parse2(attr)
        .expect_or_abort("cannot parse attribute arguments");
    let args: Vec<NestedMeta> = raw_args.into_iter().collect();

    // 2) extract our four parameters
    let mut file_path = None;
    let mut module_name = None;
    let mut yosys = None;
    let mut svql_pat_plugin_path = None;

    let workspace_root = std::env::var("CARGO_WORKSPACE_DIR").expect("workspace root not set");

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("file") => match nv.lit {
                Lit::Str(s) => {
                    file_path = Some(prepend_relative_path(&s.value(), &workspace_root));
                }
                other => abort!(other, "`file` must be a string literal"),
            },

            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("module") => match nv.lit {
                Lit::Str(s) => module_name = Some(s.value()),
                other => abort!(other, "`module` must be a string literal"),
            },

            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("yosys") => match nv.lit {
                Lit::Str(s) => {
                    // if not a path, do nothing, an executable will be searched in PATH
                    if !is_path(&s.value()) {
                        yosys = Some(s.value());
                    } else {
                        yosys = Some(prepend_relative_path(&s.value(), &workspace_root));
                    }
                }
                other => abort!(other, "`yosys` must be a string literal"),
            },

            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("svql_pat_plugin_path") => {
                match nv.lit {
                    Lit::Str(s) => {
                        svql_pat_plugin_path =
                            Some(prepend_relative_path(&s.value(), &workspace_root));
                    }
                    other => abort!(other, "`svql_pat_plugin_path` must be a string literal"),
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

    // 3) parse the item we are attached to, must be a non-generic struct
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

fn prepend_relative_path(file_path: &str, prefix: &str) -> String {
    // after you extract `let mut file_path = s.value();` from the LitStr…
    if !std::path::Path::new(&file_path).is_absolute() {
        let abs = std::path::Path::new(&prefix).join(file_path);
        abs.canonicalize()
            .unwrap_or(abs)
            .to_string_lossy()
            .into_owned()
    } else {
        file_path.to_string()
    }
}

fn is_path(file_path: &str) -> bool {
    file_path.contains(std::path::MAIN_SEPARATOR) || file_path.chars().any(std::path::is_separator)
}
