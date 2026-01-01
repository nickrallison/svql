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
use svql_query::traits::{Query, Reportable};
use tracing::{error, info};

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
    counts: Vec<QueryCount>,
}

struct QueryCount {
    query: String,
    design: String,
    count: usize,
}

trait QueryRunner: Send + Sync {
    fn run(
        &self,
        driver: &Driver,
        key: &DriverKey,
        config: &Config,
        task: &DesignTask,
        format: ResultFormat,
        file_cache: &std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<std::sync::Arc<str>, Vec<String>>>,
        >,
    ) -> Result<(Vec<MatchSummary>, usize), Box<dyn std::error::Error + Send + Sync>>;
}

struct TypedQueryRunner<Q>(std::marker::PhantomData<Q>);

impl<Q> QueryRunner for TypedQueryRunner<Q>
where
    Q: Query + Send + Sync,
    for<'a> Q::Matched<'a>: Reportable + Send,
{
    fn run(
        &self,
        driver: &Driver,
        key: &DriverKey,
        config: &Config,
        task: &DesignTask,
        format: ResultFormat,
        file_cache: &std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<std::sync::Arc<str>, Vec<String>>>,
        >,
    ) -> Result<(Vec<MatchSummary>, usize), Box<dyn std::error::Error + Send + Sync>> {
        let full_name = std::any::type_name::<Q>();
        let base_name = full_name.split('<').next().unwrap_or(full_name);
        let query_name = base_name.split("::").last().unwrap_or("Unknown");

        let design_container = driver.get_design(key).ok_or("design missing")?;

        // Map the error to a Send + Sync compatible Box
        let context = Q::context(driver, &config.needle_options)
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?
            .with_design(key.clone(), design_container);

        info!("executing query: {} on {}", query_name, task.module);
        let query_inst = Q::instantiate(Instance::root(query_name.to_lowercase()));
        let matches = query_inst.query(driver, &context, key, config);

        let mut summaries = Vec::new();
        let count = matches.len();

        for (i, m) in matches.iter().enumerate() {
            let report = m.to_report(&format!("Match #{}", i));

            if format == ResultFormat::Pretty {
                let mut cache = file_cache.lock().unwrap();
                let rendered = report.render_with_cache(&mut cache);
                println!(
                    "--- Design: {} | Query: {} ---\n{}",
                    task.module, query_name, rendered
                );
            } else {
                summaries.push(extract_summary(query_name, &task.module, report));
            }
        }

        Ok((summaries, count))
    }
}

macro_rules! query_list {
    ($($t:ty),* $(,)?) => {
        vec![
            $( Box::new(TypedQueryRunner::<$t>(std::marker::PhantomData)) as Box<dyn QueryRunner> ),*
        ]
    }
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

    info!(
        "starting collection for {} designs (parallel: {})",
        config.designs.len(),
        cfg!(feature = "parallel")
    );

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

    if format == ResultFormat::Pretty {
        println!("\n========================================");
        println!("EXECUTION SUMMARY");
        println!("========================================");
        println!("{:<20} | {:<30} | Matches", "Query", "Design");
        println!("{}", "-".repeat(65));
        for res in &results {
            for qc in &res.counts {
                println!("{:<20} | {:<30} | {}", qc.query, qc.design, qc.count);
            }
        }
        println!("========================================\n");
    }

    let mut all_summaries = Vec::new();
    for res in results {
        all_summaries.extend(res.summaries);
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

    let design_result = if task.is_raw {
        driver.get_or_load_design_raw(&task.path, &task.module)
    } else {
        driver.get_or_load_design(&task.path, &task.module, &search_config.haystack_options)
    };

    let (key, _) = match design_result {
        Ok(res) => res,
        Err(e) => {
            error!("failed to load design {}: {}", task.module, e);
            return TaskResult {
                summaries: vec![],
                counts: vec![],
            };
        }
    };

    let queries = query_list![Cwe1234<Search>, Cwe1271<Search>, Cwe1280<Search>,];
    let file_cache = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

    #[cfg(feature = "parallel")]
    let query_results: Vec<_> = queries
        .par_iter()
        .map(|q| q.run(driver, &key, &search_config, task, format, &file_cache))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let query_results: Vec<_> = queries
        .iter()
        .map(|q| q.run(driver, &key, &search_config, task, format, &file_cache))
        .collect();

    let mut summaries = Vec::new();
    let mut counts = Vec::new();

    for (idx, res) in query_results.into_iter().enumerate() {
        match res {
            Ok((s, count)) => {
                let full_name = match idx {
                    0 => "Cwe1234",
                    1 => "Cwe1271",
                    2 => "Cwe1280",
                    _ => "Unknown",
                };

                summaries.extend(s);
                counts.push(QueryCount {
                    query: full_name.to_string(),
                    design: task.module.clone(),
                    count,
                });
            }
            Err(e) => error!("query failed for {}: {}", task.module, e),
        }
    }

    TaskResult { summaries, counts }
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
