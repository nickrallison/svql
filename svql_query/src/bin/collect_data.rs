use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::sync::Arc;
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::{Driver, DriverKey};
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::report::ReportNode;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::security::cwe1271::Cwe1271;
use svql_query::security::cwe1280::Cwe1280;
use svql_query::traits::{Query, Reportable, Searchable};

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

    let file = File::open(config_path)?;
    let config: CollectionConfig = serde_json::from_reader(file)?;
    let driver = Driver::new_workspace()?;
    let mut all_summaries = Vec::new();

    for task in config.designs {
        let search_config = Config::builder()
            .match_length(MatchLength::NeedleSubsetHaystack)
            .dedupe(Dedupe::Inner)
            .max_recursion_depth(Some(task.max_depth))
            .build();

        let (key, design) = match task.is_raw {
            true => driver.get_or_load_design_raw(&task.path, &task.module)?,
            false => driver.get_or_load_design(
                &task.path,
                &task.module,
                &search_config.haystack_options,
            )?,
        };

        // Execute query suite
        all_summaries.extend(run_suite::<Cwe1234<Search>>(
            &driver,
            &key,
            &search_config,
            &task,
            format,
        )?);
        all_summaries.extend(run_suite::<Cwe1271<Search>>(
            &driver,
            &key,
            &search_config,
            &task,
            format,
        )?);
        all_summaries.extend(run_suite::<Cwe1280<Search>>(
            &driver,
            &key,
            &search_config,
            &task,
            format,
        )?);
    }

    // Final structured output dispatch
    match format {
        ResultFormat::Json => println!("{}", serde_json::to_string_pretty(&all_summaries)?),
        ResultFormat::Csv => report_csv(&all_summaries),
        ResultFormat::Pretty => (), // Already printed during run_suite
    }

    Ok(())
}

/// Executes a specific query type and collects results
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
    let context = Q::context(driver, &config.needle_options)?
        .with_design(key.clone(), driver.get_design(key).unwrap());

    let query_inst = Q::instantiate(Instance::root(query_name.to_lowercase()));
    let matches = query_inst.query(driver, &context, key, config);

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

/// Flattens a ReportNode into a structured summary without source text
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
