use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{LitStr, Path as SynPath, parse_str};
use walkdir::WalkDir;

use svql_common::build_support::{sanitize_ident, test_case_names};

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

    // Generate tests over ALL_TEST_CASES
    emit_generated_tests(&found);

    // Generate dispatch used by the CLI
    emit_generated_query_dispatch(&found);
}

fn emit_generated_tests(found: &[Discovered]) {
    let test_case_names = test_case_names();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("svql_query_generated_tests.rs");
    let mut f = File::create(&out_file).expect("Failed to create generated tests file");

    let arms: Vec<TokenStream> = found
        .iter()
        .map(|d| {
            let full_path_str = format!("{}::{}", d.module_path, d.type_name);
            let ty_path: SynPath = parse_str(&full_path_str).expect("parse discovered type path");

            // Legacy alias for composites to match existing TestCase paths:
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
                        let ctx = #ty_path::<Search>::context(driver, &tc.config).map_err(|e| e.to_string())?;
                        let (hk, hd) = driver
                            .get_or_load_design(tc.haystack.path, tc.haystack.module.to_string(), &tc.config)
                            .map_err(|e| e.to_string())?;
                        let ctx = ctx.with_design(hk.clone(), hd);
                        let root = svql_query::instance::Instance::root(tc.name.to_string());
                        let hits = #ty_path::<Search>::query(&hk, &ctx, root, &tc.config);
                        Ok(hits.len())
                    }
                }
            } else {
                quote! {
                    #primary #( | #aliases )* => {
                        let ctx = #ty_path::<Search>::context(driver, &tc.config).map_err(|e| e.to_string())?;
                        let (hk, hd) = driver
                            .get_or_load_design(tc.haystack.path, tc.haystack.module.to_string(), &tc.config)
                            .map_err(|e| e.to_string())?;
                        let ctx = ctx.with_design(hk.clone(), hd);
                        let root = svql_query::instance::Instance::root(tc.name.to_string());
                        let hits = #ty_path::<Search>::query(&hk, &ctx, root, &tc.config);
                        Ok(hits.len())
                    }
                }
            }
        })
        .collect();

    // Build the per-test functions
    let test_fns = test_case_names.iter().map(|name| {
        let fn_ident = Ident::new(
            &sanitize_ident(&format!("test_{}", name)),
            Span::call_site(),
        );
        let name_lit = name.as_str();

        quote! {
            #[test]
            fn #fn_ident() {
                init_test_logger();

                let driver = Driver::new_workspace().expect("Failed to create driver");

                let tc = ALL_TEST_CASES
                    .iter()
                    .find(|t| t.name == #name_lit)
                    .expect("TestCase not found by name");

                // Only run for test cases with a query type (netlist with Some(..) or composite)
                let query_name_opt = match tc.pattern {
                    Pattern::Netlist { pattern_query_type: Some(name), .. } => Some(name),
                    Pattern::Composite { pattern_query_type: name } => Some(name),
                    _ => None,
                };

                if let Some(name) = query_name_opt {
                    match run_count_by_type_name(name, &driver, tc) {
                        Ok(actual) => {
                            assert_eq!(
                                actual,
                                tc.expected_matches,
                                "Query test case '{}' failed: expected {} matches, got {}",
                                tc.name,
                                tc.expected_matches,
                                actual
                            );
                        }
                        Err(e) => panic!("Query test case '{}' failed: {}", tc.name, e),
                    }
                } else {
                    // Not a query-backed test case; no-op to mirror other integration tests' filtering
                }
            }
        }
    });

    let file_tokens = quote! {
        // Auto-generated by build.rs. Do not edit by hand.

        use svql_common::{ALL_TEST_CASES, Pattern};
        use svql_driver::Driver;
        use svql_query::Search;
        use svql_query::composite::{SearchableComposite, SearchableEnumComposite};
        use svql_query::netlist::SearchableNetlist;
        use tracing_subscriber;

        fn init_test_logger() {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_test_writer()
                .try_init();
        }

        // Return Ok(count) if name is known, otherwise Err("Unknown query type: ...")
        fn run_count_by_type_name(
            name: &str,
            driver: &Driver,
            tc: &svql_common::TestCase,
        ) -> Result<usize, String> {
            match name {
                #(#arms,)*
                _ => Err(format!("Unknown query type: {}", name)),
            }
        }

        #(#test_fns)*
    };

    f.write_all(file_tokens.to_string().as_bytes())
        .expect("write generated tests");
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
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::context(driver, config)),
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::query(&hk, &ctx, root, config)),
                    )
                }
                QueryKind::Composite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::context(driver, config)),
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::query(&hk, &ctx, root, config)),
                    )
                }
                QueryKind::EnumComposite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableEnumComposite>::context(driver, config)),
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
                            .get_or_load_design(haystack_path, haystack_module.to_string(), config)
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
                            .get_or_load_design(haystack_path, haystack_module.to_string(), config)
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

    // Arms for the progress-aware dispatch. For now, only netlists feed Progress into subgraph;
    // composites fallback to plain query (no updates).
    let arms_with_progress: Vec<TokenStream> = found
        .iter()
        .map(|d| {
            let full_path_str = format!("{}::{}", d.module_path, d.type_name);
            let ty_path: SynPath = parse_str(&full_path_str).expect("parse discovered type path");
            let ty_search = quote!(#ty_path::<svql_query::Search>);

            enum Kind { Netlist, Composite, EnumComposite }
            let k = match d.kind {
                QueryKind::Netlist => Kind::Netlist,
                QueryKind::Composite => Kind::Composite,
                QueryKind::EnumComposite => Kind::EnumComposite,
            };

            let (ctx_call, query_call): (TokenStream, TokenStream) = match k {
                Kind::Netlist => {
                    (
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::context(driver, config)),
                        quote!(<#ty_search as svql_query::netlist::SearchableNetlist>::query_with_progress(&hk, &ctx, root, config, progress)),
                    )
                }
                Kind::Composite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::context(driver, config)),
                        quote!(<#ty_search as svql_query::composite::SearchableComposite>::query(&hk, &ctx, root, config)),
                    )
                }
                Kind::EnumComposite => {
                    (
                        quote!(<#ty_search as svql_query::composite::SearchableEnumComposite>::context(driver, config)),
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
                            .get_or_load_design(haystack_path, haystack_module.to_string(), config)
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
                            .get_or_load_design(haystack_path, haystack_module.to_string(), config)
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
        use svql_subgraph::progress::Progress;

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

        pub fn run_count_for_type_name_with_progress(
            name: &str,
            driver: &Driver,
            haystack_path: &str,
            haystack_module: &str,
            config: &Config,
            progress: &Progress,
        ) -> Result<usize, String> {
            match name {
                #(#arms_with_progress,)*
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
