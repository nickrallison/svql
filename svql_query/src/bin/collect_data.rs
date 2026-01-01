use argparse::{ArgumentParser, Store};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
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

static PRINT_LOCK: Mutex<()> = Mutex::new(());

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
    subquery: String,
    file: String,
    lines: Vec<usize>,
}

struct MergedFinding {
    summary: MatchSummary,
}

struct TaskResult {
    findings: Vec<MergedFinding>,
    counts: Vec<QueryCount>,
}

struct QueryCount {
    query: String,
    design: String,
    count: usize,
}

trait QueryRunner: Send + Sync {
    fn name(&self) -> String;
    fn run(
        &self,
        driver: &Driver,
        key: &DriverKey,
        config: &Config,
        task: &DesignTask,
    ) -> Result<Vec<(MatchSummary, ReportNode)>, Box<dyn std::error::Error + Send + Sync>>;
}

struct TypedQueryRunner<Q>(std::marker::PhantomData<Q>);

impl<Q> QueryRunner for TypedQueryRunner<Q>
where
    Q: Query + Send + Sync,
    for<'a> Q::Matched<'a>: Reportable + Send,
{
    fn name(&self) -> String {
        let full_name = std::any::type_name::<Q>();
        let base_name = full_name.split('<').next().unwrap_or(full_name);
        base_name
            .split("::")
            .last()
            .unwrap_or("Unknown")
            .to_string()
    }

    fn run(
        &self,
        driver: &Driver,
        key: &DriverKey,
        config: &Config,
        task: &DesignTask,
    ) -> Result<Vec<(MatchSummary, ReportNode)>, Box<dyn std::error::Error + Send + Sync>> {
        let query_name = self.name();
        let design_container = driver.get_design(key).ok_or("design missing")?;
        let context = Q::context(driver, &config.needle_options)
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?
            .with_design(key.clone(), design_container);

        let matches = Q::instantiate(Instance::root(query_name.to_lowercase()))
            .query(driver, &context, key, config);

        Ok(matches
            .into_iter()
            .enumerate()
            .map(|(i, m)| {
                let report = m.to_report(&format!("Match #{}", i));
                (
                    extract_summary(&query_name, &task.module, report.clone()),
                    report,
                )
            })
            .collect())
    }
}

macro_rules! query_list {
    ($($t:ty),* $(,)?) => {
        vec![ $( Box::new(TypedQueryRunner::<$t>(std::marker::PhantomData)) as Box<dyn QueryRunner> ),* ]
    }
}

struct AppArgs {
    config_path: String,
    format: ResultFormat,
}

impl AppArgs {
    fn parse() -> Self {
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

        Self {
            config_path,
            format,
        }
    }
}

struct Collector {
    driver: Driver,
}

impl Collector {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            driver: Driver::new_workspace()?,
        })
    }

    fn run_tasks(&self, tasks: &[DesignTask]) -> Vec<TaskResult> {
        info!(
            "starting collection for {} designs (parallel: {})",
            tasks.len(),
            cfg!(feature = "parallel")
        );

        #[cfg(feature = "parallel")]
        {
            tasks
                .par_iter()
                .map(|task| self.process_design(task))
                .collect()
        }

        #[cfg(not(feature = "parallel"))]
        {
            tasks.iter().map(|task| self.process_design(task)).collect()
        }
    }

    fn process_design(&self, task: &DesignTask) -> TaskResult {
        let search_config = Config::builder()
            .match_length(MatchLength::NeedleSubsetHaystack)
            .dedupe(Dedupe::Inner)
            .max_recursion_depth(Some(task.max_depth))
            .build();

        let design_result = match task.is_raw {
            true => self.driver.get_or_load_design_raw(&task.path, &task.module),
            false => self.driver.get_or_load_design(
                &task.path,
                &task.module,
                &search_config.haystack_options,
            ),
        };

        let (key, _) = match design_result {
            Ok(res) => res,
            Err(e) => {
                error!("failed to load design {}: {}", task.module, e);
                return TaskResult {
                    findings: vec![],
                    counts: vec![],
                };
            }
        };

        let runners: Vec<Box<dyn QueryRunner>> =
            query_list![Cwe1234<Search>, Cwe1271<Search>, Cwe1280<Search>];

        #[cfg(feature = "parallel")]
        let query_results: Vec<_> = runners
            .par_iter()
            .map(|q: &Box<dyn QueryRunner>| q.run(&self.driver, &key, &search_config, task))
            .collect();

        #[cfg(not(feature = "parallel"))]
        let query_results: Vec<_> = runners
            .iter()
            .map(|q: &Box<dyn QueryRunner>| q.run(&self.driver, &key, &search_config, task))
            .collect();

        let mut final_findings = Vec::new();
        let mut final_counts = Vec::new();

        for (runner, res) in runners.iter().zip(query_results) {
            match res {
                Ok(raw_matches) => {
                    let merged = Deduplicator::merge(raw_matches);
                    final_counts.push(QueryCount {
                        query: runner.name(),
                        design: task.module.clone(),
                        count: merged.len(),
                    });
                    final_findings.extend(merged);
                }
                Err(e) => error!("query {} failed: {}", runner.name(), e),
            }
        }

        TaskResult {
            findings: final_findings,
            counts: final_counts,
        }
    }
}

struct Deduplicator;

impl Deduplicator {
    fn merge(raw_matches: Vec<(MatchSummary, ReportNode)>) -> Vec<MergedFinding> {
        if raw_matches.is_empty() {
            return vec![];
        }

        let n = raw_matches.len();
        let mut parent: Vec<usize> = (0..n).collect();
        let mut point_to_matches: HashMap<(String, usize), Vec<usize>> = HashMap::new();

        for (idx, (summary, _)) in raw_matches.iter().enumerate() {
            for loc in &summary.locations {
                for &line in &loc.lines {
                    point_to_matches
                        .entry((loc.file.clone(), line))
                        .or_default()
                        .push(idx);
                }
            }
        }

        for matches in point_to_matches.values() {
            let first = matches[0];
            matches
                .iter()
                .skip(1)
                .for_each(|&other| union_sets(first, other, &mut parent));
        }

        let mut groups: HashMap<usize, Vec<(MatchSummary, ReportNode)>> = HashMap::new();
        raw_matches.into_iter().enumerate().for_each(|(idx, pair)| {
            groups
                .entry(find_root(idx, &mut parent))
                .or_default()
                .push(pair);
        });

        groups.into_values().map(Self::merge_group).collect()
    }

    fn merge_group(mut members: Vec<(MatchSummary, ReportNode)>) -> MergedFinding {
        let (mut base_summary, _) = members.remove(0);

        for (other_summary, _) in members {
            for other_loc in other_summary.locations {
                if let Some(existing) = base_summary
                    .locations
                    .iter_mut()
                    .find(|l| l.subquery == other_loc.subquery && l.file == other_loc.file)
                {
                    existing.lines.extend(other_loc.lines);
                    existing.lines.sort();
                    existing.lines.dedup();
                } else {
                    base_summary.locations.push(other_loc);
                }
            }
        }
        MergedFinding {
            summary: base_summary,
        }
    }
}

struct Reporter {
    format: ResultFormat,
    file_cache: Arc<Mutex<HashMap<Arc<str>, Vec<String>>>>,
}

impl Reporter {
    fn new(format: ResultFormat) -> Self {
        Self {
            format,
            file_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn render(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
        match self.format {
            ResultFormat::Pretty => self.render_pretty(results),
            ResultFormat::Json => self.render_json(results),
            ResultFormat::Csv => self.render_csv(results),
        }
    }

    fn render_pretty(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
        let _lock = PRINT_LOCK.lock().unwrap();
        for res in results {
            for finding in &res.findings {
                println!(
                    "\n=== Merged Finding: {} | {} ===",
                    finding.summary.query, finding.summary.design
                );
                println!("Instance Path: {}", finding.summary.instance_path);

                let mut by_file: std::collections::HashMap<String, Vec<&Location>> =
                    std::collections::HashMap::new();

                for loc in &finding.summary.locations {
                    by_file.entry(loc.file.clone()).or_default().push(loc);
                }

                let mut cache = self.file_cache.lock().unwrap();

                for (file_path, mut locs) in by_file {
                    println!("Source File: {}", file_path);

                    let file_lines = cache
                        .entry(std::sync::Arc::from(file_path.as_str()))
                        .or_insert_with(|| read_file_lines(&file_path).unwrap_or_default());

                    locs.sort_by_key(|l| &l.subquery);

                    for loc in locs {
                        let ranges = format_line_ranges(&loc.lines);
                        println!("  [Sub-component: {}] Lines: {}", loc.subquery, ranges);

                        for &line_num in &loc.lines {
                            if line_num > 0 && line_num <= file_lines.len() {
                                let content = &file_lines[line_num - 1];
                                println!("    {:>4} | {}", line_num, content.trim_end());
                            } else {
                                println!("    {:>4} | <line not found>", line_num);
                            }
                        }
                    }
                }
            }
        }

        println!(
            "\n========================================\nEXECUTION SUMMARY\n========================================"
        );
        println!(
            "{:<20} | {:<30} | Matches\n{}",
            "Query",
            "Design",
            "-".repeat(65)
        );
        for res in results {
            for qc in &res.counts {
                println!("{:<20} | {:<30} | {}", qc.query, qc.design, qc.count);
            }
        }
        Ok(())
    }

    fn render_json(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
        let summaries: Vec<_> = results
            .iter()
            .flat_map(|r| r.findings.iter().map(|f| &f.summary))
            .collect();
        let _lock = PRINT_LOCK.lock().unwrap();
        println!("{}", serde_json::to_string_pretty(&summaries)?);
        Ok(())
    }

    fn render_csv(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
        let summaries: Vec<_> = results
            .iter()
            .flat_map(|r| r.findings.iter().map(|f| &f.summary))
            .collect();
        let _lock = PRINT_LOCK.lock().unwrap();
        println!("query,design,instance_path,locations");
        for r in summaries {
            let mut locs: Vec<_> = r.locations.iter().collect();
            locs.sort_by_key(|l| &l.subquery);

            let flat_locs = locs
                .iter()
                .map(|l| {
                    format!(
                        "{}@{}:[{}]",
                        l.subquery,
                        l.file,
                        format_line_ranges(&l.lines)
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");

            println!(
                "{},{},{},\"{}\"",
                r.query, r.design, r.instance_path, flat_locs
            );
        }
        Ok(())
    }
}

fn read_file_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

fn find_root(i: usize, parent: &mut [usize]) -> usize {
    if parent[i] == i {
        i
    } else {
        parent[i] = find_root(parent[i], parent);
        parent[i]
    }
}

fn union_sets(i: usize, j: usize, parent: &mut [usize]) {
    let root_i = find_root(i, parent);
    let root_j = find_root(j, parent);
    if root_i != root_j {
        parent[root_i] = root_j;
    }
}

fn format_line_ranges(lines: &[usize]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let start = lines[i];
        let mut end = start;
        while i + 1 < lines.len() && lines[i + 1] == end + 1 {
            i += 1;
            end = lines[i];
        }
        if start == end {
            ranges.push(start.to_string());
        } else {
            ranges.push(format!("{}-{}", start, end));
        }
        i += 1;
    }
    ranges.join(",")
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
            subquery: node.name.clone(),
            file: node.source_loc.file.to_string(),
            lines: node.source_loc.lines.iter().map(|l| l.number).collect(),
        });
    }
    node.children
        .iter()
        .for_each(|child| collect_locations(child, set));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = AppArgs::parse();
    let config: CollectionConfig = serde_json::from_reader(File::open(&args.config_path)?)?;

    let collector = Collector::new()?;
    let results = collector.run_tasks(&config.designs);

    let reporter = Reporter::new(args.format);
    reporter.render(&results)?;

    Ok(())
}
