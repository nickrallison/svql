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
use tracing::{debug, error, info};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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

struct TaskResult {
    summaries: Vec<MatchSummary>,
    pretty_output: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use argparse::{ArgumentParser, Store};

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

    let file = File::open(&config_path)?;
    let config: CollectionConfig = serde_json::from_reader(file)?;
    let driver = Driver::new_workspace()?;

    #[cfg(feature = "parallel")]
    info!(
        "starting parallel collection for {} designs",
        config.designs.len()
    );
    #[cfg(not(feature = "parallel"))]
    info!(
        "starting sequential collection for {} designs",
        config.designs.len()
    );

    // Feature gated parallel iteration across designs
    #[cfg(feature = "parallel")]
    let results: Vec<TaskResult> = config
        .designs
        .par_iter()
        .map(|task| process_design_task(&driver, task, format))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let results: Vec<TaskResult> = config
        .designs
        .iter()
        .map(|task| process_design_task(&driver, task, format))
        .collect();

    let mut all_summaries = Vec::new();
    for res in results {
        all_summaries.extend(res.summaries);
        for output in res.pretty_output {
            println!("{}", output);
        }
    }

    match format {
        ResultFormat::Json => println!("{}", serde_json::to_string_pretty(&all_summaries)?),
        ResultFormat::Csv => report_csv(&all_summaries),
        ResultFormat::Pretty => (),
    }

    Ok(())
}

fn process_design_task(driver: &Driver, task: &DesignTask, format: ResultFormat) -> TaskResult {
    info!("processing design: {} ({})", task.module, task.path);

    let search_config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::Inner)
        .max_recursion_depth(Some(task.max_depth))
        .build();

    let design_result = match task.is_raw {
        true => driver.get_or_load_design_raw(&task.path, &task.module),
        false => {
            driver.get_or_load_design(&task.path, &task.module, &search_config.haystack_options)
        }
    };

    let (key, _) = match design_result {
        Ok(res) => res,
        Err(e) => {
            error!("failed to load design {}: {}", task.module, e);
            return TaskResult {
                summaries: vec![],
                pretty_output: vec![],
            };
        }
    };

    // Feature gated parallel query execution
    #[cfg(feature = "parallel")]
    let (r1, (r2, r3)) = rayon::join(
        || run_suite::<Cwe1234<Search>>(driver, &key, &search_config, task, format),
        || {
            rayon::join(
                || run_suite::<Cwe1271<Search>>(driver, &key, &search_config, task, format),
                || run_suite::<Cwe1280<Search>>(driver, &key, &search_config, task, format),
            )
        },
    );

    #[cfg(not(feature = "parallel"))]
    let (r1, r2, r3) = (
        run_suite::<Cwe1234<Search>>(driver, &key, &search_config, task, format),
        run_suite::<Cwe1271<Search>>(driver, &key, &search_config, task, format),
        run_suite::<Cwe1280<Search>>(driver, &key, &search_config, task, format),
    );

    let mut summaries = Vec::new();
    let mut pretty_output = Vec::new();

    for res in [r1, r2, r3] {
        match res {
            Ok((s, p)) => {
                summaries.extend(s);
                pretty_output.extend(p);
            }
            Err(e) => error!("query failed for {}: {}", task.module, e),
        }
    }

    TaskResult {
        summaries,
        pretty_output,
    }
}

fn run_suite<Q>(
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
    task: &DesignTask,
    format: ResultFormat,
) -> Result<(Vec<MatchSummary>, Vec<String>), Box<dyn std::error::Error + Send + Sync>>
where
    Q: Query + Searchable + Send + Sync,
    for<'a> Q::Matched<'a>: Reportable + Send,
{
    let query_name = std::any::type_name::<Q>()
        .split("::")
        .last()
        .unwrap_or("Unknown");

    let design_container = driver.get_design(key).ok_or_else(|| {
        error!("design key not found: {:?}", key);
        "design missing"
    })?;

    let context = Q::context(driver, &config.needle_options)
        .map_err(|e| {
            error!("context error: {}", e);
            "context error"
        })?
        .with_design(key.clone(), design_container);

    info!("executing query: {} on {}", query_name, task.module);
    let query_inst = Q::instantiate(Instance::root(query_name.to_lowercase()));
    let matches = query_inst.query(driver, &context, key, config);

    let mut summaries = Vec::new();
    let mut pretty_strings = Vec::new();

    for (i, m) in matches.iter().enumerate() {
        let report = m.to_report(&format!("Match #{}", i));

        match format {
            ResultFormat::Pretty => {
                let mut out = format!("--- Design: {} | Query: {} ---\n", task.module, query_name);
                out.push_str(&report.render());
                pretty_strings.push(out);
            }
            _ => summaries.push(extract_summary(query_name, &task.module, report)),
        }
    }

    Ok((summaries, pretty_strings))
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
