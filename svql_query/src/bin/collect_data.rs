use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::{Driver, DriverKey};
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::report::ReportNode;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::security::cwe1271::Cwe1271;
use svql_query::security::cwe1280::Cwe1280;
use svql_query::traits::{Query, Reportable, Searchable};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultFormat {
    Json,
    Csv,
    Pretty,
}

#[derive(Deserialize, Debug)]
struct DesignTask {
    path: String,
    module: String,
    max_depth: usize,
    is_raw: bool,
}

#[derive(Deserialize, Debug)]
struct CollectionConfig {
    designs: Vec<DesignTask>,
}

#[derive(Serialize, Debug)]
struct MatchSummary {
    query: String,
    design: String,
    instance_path: String,
    locations: Vec<Location>,
}

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone)]
struct Location {
    file: String,
    lines: Vec<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use argparse::{ArgumentParser, Store};

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut config_path = String::new();
    let mut format_str = String::from("json");

    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config_path)
            .add_option(&["-c", "--config"], Store, "Path to input JSON config")
            .required();
        ap.refer(&mut format_str).add_option(
            &["-f", "--format"],
            Store,
            "Output format (json, csv, pretty)",
        );
        ap.parse_args_or_exit();
    }

    let format = match format_str.to_lowercase().as_str() {
        "csv" => ResultFormat::Csv,
        "pretty" => ResultFormat::Pretty,
        _ => ResultFormat::Json,
    };

    info!("loading collection configuration from: {}", config_path);
    let file = File::open(&config_path).map_err(|e| {
        error!("failed to open config file: {}", e);
        e
    })?;

    let config: CollectionConfig = serde_json::from_reader(file)?;
    let driver = Driver::new_workspace()?;
    let mut all_summaries = Vec::new();

    info!(
        "starting collection for {} design tasks",
        config.designs.len()
    );

    for (idx, task) in config.designs.iter().enumerate() {
        info!(
            "[{}/{}] processing design: {} ({})",
            idx + 1,
            config.designs.len(),
            task.module,
            task.path
        );

        let search_config = Config::builder()
            .match_length(MatchLength::NeedleSubsetHaystack)
            .dedupe(Dedupe::Inner)
            .max_recursion_depth(Some(task.max_depth))
            .build();

        let design_result = match task.is_raw {
            true => {
                debug!("performing raw import for {}", task.module);
                driver.get_or_load_design_raw(&task.path, &task.module)
            }
            false => {
                debug!("performing yosys import for {}", task.module);
                driver.get_or_load_design(&task.path, &task.module, &search_config.haystack_options)
            }
        };

        let (key, design) = match design_result {
            Ok(res) => res,
            Err(e) => {
                error!("failed to load design {}: {}", task.module, e);
                continue;
            }
        };

        // Execute query suite
        match run_suite_all(&driver, &key, &search_config, task, format) {
            Ok(summaries) => all_summaries.extend(summaries),
            Err(e) => error!("query suite failed for {}: {}", task.module, e),
        }
    }

    info!(
        "collection complete. total matches found: {}",
        all_summaries.len()
    );

    match format {
        ResultFormat::Json => println!("{}", serde_json::to_string_pretty(&all_summaries)?),
        ResultFormat::Csv => report_csv(&all_summaries),
        ResultFormat::Pretty => (),
    }

    Ok(())
}

/// Helper to run all registered queries
fn run_suite_all(
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
    task: &DesignTask,
    format: ResultFormat,
) -> Result<Vec<MatchSummary>, Box<dyn std::error::Error>> {
    let mut summaries = Vec::new();

    summaries.extend(run_suite::<Cwe1234<Search>>(
        driver, key, config, task, format,
    )?);
    summaries.extend(run_suite::<Cwe1271<Search>>(
        driver, key, config, task, format,
    )?);
    summaries.extend(run_suite::<Cwe1280<Search>>(
        driver, key, config, task, format,
    )?);

    Ok(summaries)
}

fn run_suite<Q>(
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
    task: &DesignTask,
    format: ResultFormat,
) -> Result<Vec<MatchSummary>, Box<dyn std::error::Error>>
where
    Q: Query + Searchable,
    for<'a> Q::Matched<'a>: Reportable,
{
    let query_name = std::any::type_name::<Q>()
        .split("::")
        .last()
        .unwrap_or("Unknown");
    debug!("preparing context for query: {}", query_name);

    let design_container = driver.get_design(key).ok_or_else(|| {
        let err = format!("design key not found in driver registry: {:?}", key);
        error!("{}", err);
        err
    })?;

    let context =
        Q::context(driver, &config.needle_options)?.with_design(key.clone(), design_container);

    info!("executing query: {} on {}", query_name, task.module);
    let query_inst = Q::instantiate(Instance::root(query_name.to_lowercase()));
    let matches = query_inst.query(driver, &context, key, config);

    if matches.is_empty() {
        debug!("no matches found for {} on {}", query_name, task.module);
    } else {
        info!(
            "found {} matches for {} on {}",
            matches.len(),
            query_name,
            task.module
        );
    }

    let mut summaries = Vec::new();
    for (i, m) in matches.iter().enumerate() {
        let report = m.to_report(&format!("Match #{}", i));

        match format {
            ResultFormat::Pretty => {
                println!("--- Design: {} | Query: {} ---", task.module, query_name);
                println!("{}", report.render());
            }
            _ => summaries.push(extract_summary(query_name, &task.module, report)),
        }
    }

    Ok(summaries)
}

fn extract_summary(query: &str, design: &str, node: ReportNode) -> MatchSummary {
    let mut locations = HashSet::new();
    collect_locations(&node, &mut locations);

    MatchSummary {
        query: query.to_string(),
        design: design.to_string(),
        instance_path: node.path.inst_path(),
        locations: locations.into_iter().collect(),
    }
}

fn collect_locations(node: &ReportNode, set: &mut HashSet<Location>) {
    if !node.source_loc.lines.is_empty() {
        set.insert(Location {
            file: node.source_loc.file.to_string(),
            lines: node.source_loc.lines.iter().map(|l| l.number).collect(),
        });
    }
    for child in &node.children {
        collect_locations(child, set);
    }
}

fn report_csv(results: &[MatchSummary]) {
    println!("query,design,instance_path,file,lines");
    for r in results {
        for loc in &r.locations {
            let lines = loc
                .lines
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(";");
            println!(
                "{},{},{},{},\"{}\"",
                r.query, r.design, r.instance_path, loc.file, lines
            );
        }
    }
}
