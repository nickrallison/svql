use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{LitStr, Path as SynPath, parse_str};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum QueryKind {
    Netlist,
    Composite,
    EnumComposite,
}

#[derive(Debug, Clone)]
struct Discovered {
    kind: QueryKind,
    type_name: String,   // e.g., "AndGate"
    module_path: String, // e.g., "svql_query::queries::netlist::basic::and"
}

fn main() {
    // Re-run if these change
    println!("cargo:rerun-if-changed=src/queries");
    println!("cargo:rerun-if-changed=../svql_common/src/test_cases.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_queries = manifest_dir.join("src").join("queries");

    let mut found: Vec<Discovered> = discover_query_types(&src_queries);
    found.sort_by(|a, b| {
        (format!("{:?}", a.kind), &a.type_name, &a.module_path).cmp(&(
            format!("{:?}", b.kind),
            &b.type_name,
            &b.module_path,
        ))
    });
    found.dedup_by(|a, b| {
        a.kind == b.kind && a.type_name == b.type_name && a.module_path == b.module_path
    });

    // Generate dispatch used by the CLI
    emit_generated_query_dispatch(&found);
}

fn emit_generated_query_dispatch(found: &[Discovered]) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("svql_query_query_dispatch.rs");
    let mut f = File::create(&out_file).expect("Failed to create dispatch file");

    // Arms for dispatch using UFCS with the correct trait per kind
    let arms_plain: Vec<TokenStream> = found
        .iter()
        .map(|d| {
            let full_path_str = format!("{}::{}", d.module_path, d.type_name);
            let ty_path: SynPath = parse_str(&full_path_str).expect("parse discovered type path");

            let ty_search = quote!(#ty_path::<svql_query::Search>);

            let (ctx_call, query_call): (TokenStream, TokenStream) = match d.kind {
                QueryKind::Netlist => {
                    (
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::context(driver, &config.needle_options)),
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::query(&hk, &ctx, root, config)),
                    )
                }
                QueryKind::Composite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::context(driver, &config.needle_options)),
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::query(&hk, &ctx, root, config)),
                    )
                }
                QueryKind::EnumComposite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableEnumComposite>::context(driver, &config.needle_options)),
                        quote!(<#ty_search as svql_query::composite::SearchableEnumComposite>::query(&hk, &ctx, root, config)),
                    )
                }
            };

            let aliases: Vec<LitStr> = match d.kind {
                QueryKind::Composite | QueryKind::EnumComposite => {
                    vec![LitStr::new(
                        &format!("svql_query::queries::netlist::composite::{}", d.type_name),
                        Span::call_site(),
                    )]
                }
                _ => vec![],
            };

            let primary = LitStr::new(&full_path_str, Span::call_site());

            if aliases.is_empty() {
                quote! {
                    #primary => {
                        let ctx = #ctx_call.map_err(|e| e.to_string())?;
                        let (hk, hd) = driver
                            .get_or_load_design(haystack_path, &haystack_module.to_string(), &config.haystack_options)
                            .map_err(|e| e.to_string())?;
                        let ctx = ctx.with_design(hk.clone(), hd);
                        let root = svql_query::instance::Instance::root("cli_root".to_string());
                        let hits = #query_call;
                        Ok(hits.len())
                    }
                }
            } else {
                quote! {
                    #primary #( | #aliases )* => {
                        let ctx = #ctx_call.map_err(|e| e.to_string())?;
                        let (hk, hd) = driver
                            .get_or_load_design(haystack_path, &haystack_module.to_string(), &config.haystack_options)
                            .map_err(|e| e.to_string())?;
                        let ctx = ctx.with_design(hk.clone(), hd);
                        let root = svql_query::instance::Instance::root("cli_root".to_string());
                        let hits = #query_call;
                        Ok(hits.len())
                    }
                }
            }
        })
        .collect();

    let mut names: Vec<String> = Vec::new();
    for d in found {
        let p = format!("{}::{}", d.module_path, d.type_name);
        names.push(p);
        if matches!(d.kind, QueryKind::Composite | QueryKind::EnumComposite) {
            names.push(format!(
                "svql_query::queries::netlist::composite::{}",
                d.type_name
            ));
        }
    }
    names.sort();
    names.dedup();

    let names_lits: Vec<LitStr> = names
        .iter()
        .map(|s| LitStr::new(s, Span::call_site()))
        .collect();

    let file_tokens = quote! {
        // Auto-generated by build.rs. Do not edit by hand.

        use svql_common::Config;
        use svql_driver::Driver;

        pub fn run_count_for_type_name(
            name: &str,
            driver: &Driver,
            haystack_path: &str,
            haystack_module: &str,
            config: &Config,
        ) -> Result<usize, String> {
            match name {
                #(#arms_plain,)*
                _ => Err(format!("Unknown query type: {}", name)),
            }
        }

        pub fn known_query_type_names() -> &'static [&'static str] {
            &[
                #(#names_lits),*
            ]
        }
    };

    f.write_all(file_tokens.to_string().as_bytes())
        .expect("write generated dispatch");
}

fn discover_query_types(src_queries: &Path) -> Vec<Discovered> {
    let re_netlist = Regex::new(r#"netlist!\s*\{\s*name:\s*([A-Za-z_][A-Za-z0-9_]*)"#).unwrap();
    let re_composite = Regex::new(
        r#"impl\s+SearchableComposite\s+for\s+([A-Za-z_][A-Za-z0-9_]*)\s*<\s*Search\s*>"#,
    )
    .unwrap();
    let re_enum_composite = Regex::new(
        r#"impl\s+SearchableEnumComposite\s+for\s+([A-Za-z_][A-Za-z0-9_]*)\s*<\s*Search\s*>"#,
    )
    .unwrap();

    let mut found = Vec::new();

    for entry in WalkDir::new(src_queries).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let module_path = path_to_module_path(src_queries, path);

        for cap in re_netlist.captures_iter(&content) {
            found.push(Discovered {
                kind: QueryKind::Netlist,
                type_name: cap[1].to_string(),
                module_path: module_path.clone(),
            });
        }

        for cap in re_composite.captures_iter(&content) {
            found.push(Discovered {
                kind: QueryKind::Composite,
                type_name: cap[1].to_string(),
                module_path: module_path.clone(),
            });
        }

        for cap in re_enum_composite.captures_iter(&content) {
            found.push(Discovered {
                kind: QueryKind::EnumComposite,
                type_name: cap[1].to_string(),
                module_path: module_path.clone(),
            });
        }
    }

    found
}

// Convert src/queries/netlist/basic/and.rs -> svql_query::queries::netlist::basic::and
fn path_to_module_path(base: &Path, file: &Path) -> String {
    let mut comps: Vec<String> = Vec::new();
    comps.push("svql_query".to_string());
    comps.push("queries".to_string());

    let under_queries = file.strip_prefix(base).unwrap_or(file);
    for c in under_queries
        .parent()
        .into_iter()
        .flat_map(|p| p.components())
    {
        let s = c.as_os_str().to_string_lossy().to_string();
        comps.push(s);
    }

    if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
        comps.push(stem.to_string());
    }
    comps.join("::")
}
