// use svql_query::prelude::*;

// use argparse::{ArgumentParser, Store};
// use serde::{Deserialize, Serialize};
// use std::collections::{HashMap, HashSet};
// use std::fs::File;
// use std::io::{BufRead, BufReader};
// use std::sync::{Arc, Mutex};
// use std::time::Instant;
// use svql_query::security::cwe1271::Cwe1271;
// use svql_query::security::cwe1280::Cwe1280;
// use sysinfo::{ProcessRefreshKind, System};
// use tracing::{error, info};

// #[cfg(feature = "parallel")]
// use rayon::prelude::*;

// static PRINT_LOCK: Mutex<()> = Mutex::new(());

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum ResultFormat {
//     Json,
//     Csv,
//     Pretty,
// }

// #[derive(Deserialize, Debug)]
// struct DesignTask {
//     path: String,
//     module: String,
//     max_depth: usize,
//     is_raw: bool,
// }

// #[derive(Deserialize, Debug)]
// struct CollectionConfig {
//     designs: Vec<DesignTask>,
// }

// #[derive(Serialize, Debug, Clone)]
// struct PerformanceMetrics {
//     design_name: String,
//     gate_count: usize,
//     query_times_ms: HashMap<String, u128>,
//     peak_rss_kb: u64,
// }

// #[derive(Serialize, Debug)]
// struct MatchSummary {
//     query: String,
//     design: String,
//     instance_path: String,
//     locations: Vec<Location>,
// }

// #[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone)]
// struct Location {
//     subquery: String,
//     file: String,
//     lines: Vec<usize>,
// }

// struct MergedFinding {
//     summary: MatchSummary,
// }

// struct TaskResult {
//     findings: Vec<MergedFinding>,
//     counts: Vec<QueryCount>,
//     metrics: PerformanceMetrics,
// }

// struct QueryCount {
//     query: String,
//     design: String,
//     count: usize,
// }

// trait QueryRunner: Send + Sync {
//     fn name(&self) -> String;
//     fn run(
//         &self,
//         driver: &Driver,
//         key: &DriverKey,
//         config: &Config,
//         task: &DesignTask,
//     ) -> Result<(Vec<(MatchSummary, ReportNode)>, u128), Box<dyn std::error::Error + Send + Sync>>;
// }

// struct TypedQueryRunner<Q>(std::marker::PhantomData<Q>);

// impl<Q> QueryRunner for TypedQueryRunner<Q>
// where
//     Q: Pattern + Send + Sync + 'static,
//     Q::Match: Hardware + Send,
// {
//     fn name(&self) -> String {
//         let full_name = std::any::type_name::<Q>();
//         let base_name = full_name.split('<').next().unwrap_or(full_name);
//         base_name
//             .split("::")
//             .last()
//             .unwrap_or("Unknown")
//             .to_string()
//     }

//     fn run(
//         &self,
//         driver: &Driver,
//         key: &DriverKey,
//         config: &Config,
//         task: &DesignTask,
//     ) -> Result<(Vec<(MatchSummary, ReportNode)>, u128), Box<dyn std::error::Error + Send + Sync>>
//     {
//         let query_name = self.name();

//         let start = Instant::now();

//         let matches = execute_query::<Q>(driver, key, config)
//             .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;

//         let duration = start.elapsed().as_millis();

//         let results = matches
//             .into_iter()
//             .enumerate()
//             .map(|(i, m)| {
//                 let report = m.report(&format!("Match #{}", i));
//                 (
//                     extract_summary(&query_name, &task.module, report.clone()),
//                     report,
//                 )
//             })
//             .collect();

//         Ok((results, duration))
//     }
// }

// macro_rules! query_list {
//     ($($t:ty),* $(,)?) => {
//         vec![ $( Box::new(TypedQueryRunner::<$t>(std::marker::PhantomData)) as Box<dyn QueryRunner> ),* ]
//     }
// }

// struct AppArgs {
//     config_path: String,
//     format: ResultFormat,
//     query_name: String,
// }

// impl AppArgs {
//     fn parse() -> Self {
//         let mut config_path = String::new();
//         let mut format_str = String::from("json");
//         let mut query_name = String::new();

//         {
//             let mut ap = ArgumentParser::new();
//             ap.refer(&mut config_path)
//                 .add_option(&["-c", "--config"], Store, "Path to input JSON config")
//                 .required();
//             ap.refer(&mut query_name)
//                 .add_option(&["-q", "--query"], Store, "Specific query to run")
//                 .required();
//             ap.refer(&mut format_str).add_option(
//                 &["-f", "--format"],
//                 Store,
//                 "Output format (json, csv, pretty)",
//             );
//             ap.parse_args_or_exit();
//         }

//         let format = match format_str.to_lowercase().as_str() {
//             "csv" => ResultFormat::Csv,
//             "pretty" => ResultFormat::Pretty,
//             _ => ResultFormat::Json,
//         };

//         Self {
//             config_path,
//             format,
//             query_name,
//         }
//     }
// }

// struct Collector {
//     sys: Mutex<System>,
// }

// impl Collector {
//     fn new() -> Result<Self, Box<dyn std::error::Error>> {
//         Ok(Self {
//             sys: Mutex::new(System::new_all()),
//         })
//     }

//     fn get_rss_kb(&self) -> u64 {
//         let mut sys = self.sys.lock().unwrap();
//         let pid = sysinfo::get_current_pid().expect("Failed to get PID");
//         let refresh_kind = ProcessRefreshKind::nothing().with_memory();

//         sys.refresh_processes_specifics(
//             sysinfo::ProcessesToUpdate::Some(&[pid]),
//             true,
//             refresh_kind,
//         );

//         sys.process(pid).map(|p| p.memory() / 1024).unwrap_or(0)
//     }

//     fn run_tasks(&self, tasks: &[DesignTask], query_filter: &str) -> Vec<TaskResult> {
//         info!(
//             "starting collection for {} designs (query: {})",
//             tasks.len(),
//             query_filter
//         );

//         #[cfg(feature = "parallel")]
//         {
//             tasks
//                 .par_iter()
//                 .map(|task| self.process_design(task, query_filter))
//                 .collect()
//         }

//         #[cfg(not(feature = "parallel"))]
//         {
//             tasks
//                 .iter()
//                 .map(|task| self.process_design(task, query_filter))
//                 .collect()
//         }
//     }

//     fn process_design(&self, task: &DesignTask, query_filter: &str) -> TaskResult {
//         let (findings, counts, mut metrics) = self.run_time_pass(task, query_filter);
//         metrics.peak_rss_kb = self.run_memory_pass(task, query_filter);

//         TaskResult {
//             findings,
//             counts,
//             metrics,
//         }
//     }

//     fn get_filtered_runners(&self, filter: &str) -> Vec<Box<dyn QueryRunner>> {
//         let all_runners: Vec<Box<dyn QueryRunner>> = query_list![Cwe1271, Cwe1280];

//         all_runners
//             .into_iter()
//             .filter(|r| r.name().to_lowercase() == filter.to_lowercase())
//             .collect()
//     }

//     fn run_time_pass(
//         &self,
//         task: &DesignTask,
//         query_filter: &str,
//     ) -> (Vec<MergedFinding>, Vec<QueryCount>, PerformanceMetrics) {
//         let driver = Driver::new_workspace().expect("Failed to create driver");
//         let search_config = Config::builder()
//             .match_length(MatchLength::NeedleSubsetHaystack)
//             .dedupe(Dedupe::Inner)
//             .max_recursion_depth(Some(task.max_depth))
//             .build();

//         let design_result = match task.is_raw {
//             true => driver.get_or_load_design_raw(&task.path, &task.module),
//             false => {
//                 driver.get_or_load_design(&task.path, &task.module, &search_config.haystack_options)
//             }
//         };

//         let (key, design_container) = design_result.expect("failed to load design");
//         let gate_count = design_container.index().cells_topo().len();

//         let runners = self.get_filtered_runners(query_filter);
//         if runners.is_empty() {
//             panic!("Query {} not found in registry", query_filter);
//         }

//         #[cfg(feature = "parallel")]
//         let query_results: Vec<_> = runners
//             .par_iter()
//             .map(|q| q.run(&driver, &key, &search_config, task))
//             .collect();

//         #[cfg(not(feature = "parallel"))]
//         let query_results: Vec<_> = runners
//             .iter()
//             .map(|q| q.run(&driver, &key, &search_config, task))
//             .collect();

//         let mut final_findings = Vec::new();
//         let mut final_counts = Vec::new();
//         let mut query_times = HashMap::new();

//         for (runner, res) in runners.iter().zip(query_results) {
//             match res {
//                 Ok((raw_matches, duration)) => {
//                     query_times.insert(runner.name(), duration);
//                     let merged = Deduplicator::merge(raw_matches);

//                     let filtered_findings: Vec<_> = merged
//                         .into_iter()
//                         .filter(|f| !f.summary.locations.is_empty())
//                         .collect();

//                     final_counts.push(QueryCount {
//                         query: runner.name(),
//                         design: task.module.clone(),
//                         count: filtered_findings.len(),
//                     });
//                     final_findings.extend(filtered_findings);
//                 }
//                 Err(e) => error!("query {} failed: {}", runner.name(), e),
//             }
//         }

//         let metrics = PerformanceMetrics {
//             design_name: task.module.clone(),
//             gate_count,
//             query_times_ms: query_times,
//             peak_rss_kb: 0,
//         };

//         (final_findings, final_counts, metrics)
//     }

//     fn run_memory_pass(&self, task: &DesignTask, query_filter: &str) -> u64 {
//         let driver = Driver::new_workspace().expect("Failed to create driver");
//         let search_config = Config::builder()
//             .match_length(MatchLength::NeedleSubsetHaystack)
//             .dedupe(Dedupe::Inner)
//             .max_recursion_depth(Some(task.max_depth))
//             .build();

//         let (key, _) = driver
//             .get_or_load_design(&task.path, &task.module, &search_config.haystack_options)
//             .expect("Failed to load design");

//         let runners = self.get_filtered_runners(query_filter);
//         let mut peak = self.get_rss_kb();

//         for runner in runners {
//             let _ = runner.run(&driver, &key, &search_config, task);
//             peak = peak.max(self.get_rss_kb());
//         }

//         peak
//     }
// }

// struct Deduplicator;

// impl Deduplicator {
//     fn merge(raw_matches: Vec<(MatchSummary, ReportNode)>) -> Vec<MergedFinding> {
//         if raw_matches.is_empty() {
//             return vec![];
//         }

//         let n = raw_matches.len();
//         let mut parent: Vec<usize> = (0..n).collect();
//         let mut point_to_matches: HashMap<(String, usize), Vec<usize>> = HashMap::new();

//         for (idx, (summary, _)) in raw_matches.iter().enumerate() {
//             for loc in &summary.locations {
//                 for &line in &loc.lines {
//                     point_to_matches
//                         .entry((loc.file.clone(), line))
//                         .or_default()
//                         .push(idx);
//                 }
//             }
//         }

//         for matches in point_to_matches.values() {
//             let first = matches[0];
//             matches
//                 .iter()
//                 .skip(1)
//                 .for_each(|&other| union_sets(first, other, &mut parent));
//         }

//         let mut groups: HashMap<usize, Vec<(MatchSummary, ReportNode)>> = HashMap::new();
//         raw_matches.into_iter().enumerate().for_each(|(idx, pair)| {
//             groups
//                 .entry(find_root(idx, &mut parent))
//                 .or_default()
//                 .push(pair);
//         });

//         groups.into_values().map(Self::merge_group).collect()
//     }

//     fn merge_group(mut members: Vec<(MatchSummary, ReportNode)>) -> MergedFinding {
//         let (mut base_summary, _) = members.remove(0);

//         for (other_summary, _) in members {
//             for other_loc in other_summary.locations {
//                 if let Some(existing) = base_summary
//                     .locations
//                     .iter_mut()
//                     .find(|l| l.subquery == other_loc.subquery && l.file == other_loc.file)
//                 {
//                     existing.lines.extend(other_loc.lines);
//                     existing.lines.sort();
//                     existing.lines.dedup();
//                 } else {
//                     base_summary.locations.push(other_loc);
//                 }
//             }
//         }
//         MergedFinding {
//             summary: base_summary,
//         }
//     }
// }

// struct Reporter {
//     format: ResultFormat,
//     file_cache: Arc<Mutex<HashMap<Arc<str>, Vec<String>>>>,
// }

// impl Reporter {
//     fn new(format: ResultFormat) -> Self {
//         Self {
//             format,
//             file_cache: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }

//     fn render(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
//         match self.format {
//             ResultFormat::Pretty => self.render_pretty(results),
//             ResultFormat::Json => self.render_json(results),
//             ResultFormat::Csv => self.render_csv(results),
//         }
//     }

//     fn render_pretty(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
//         let _lock = PRINT_LOCK.lock().unwrap();
//         for res in results {
//             for finding in &res.findings {
//                 println!(
//                     "\n=== Finding: {} | {} ===",
//                     finding.summary.query, finding.summary.design
//                 );
//                 println!("Instance Path: {}", finding.summary.instance_path);

//                 let mut by_file: HashMap<String, Vec<&Location>> = HashMap::new();
//                 for loc in &finding.summary.locations {
//                     by_file.entry(loc.file.clone()).or_default().push(loc);
//                 }

//                 let mut cache = self.file_cache.lock().unwrap();
//                 for (file_path, locs) in by_file {
//                     println!("Source File: {}", file_path);
//                     let file_lines = cache
//                         .entry(Arc::from(file_path.as_str()))
//                         .or_insert_with(|| read_file_lines(&file_path).unwrap_or_default());

//                     for loc in locs {
//                         let ranges = format_line_ranges(&loc.lines);
//                         println!("  [Sub-component: {}] Lines: {}", loc.subquery, ranges);
//                         for &line_num in &loc.lines {
//                             if line_num > 0 && line_num <= file_lines.len() {
//                                 println!(
//                                     "    {:>4} | {}",
//                                     line_num,
//                                     file_lines[line_num - 1].trim_end()
//                                 );
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         println!(
//             "\n========================================\nPERFORMANCE SUMMARY\n========================================"
//         );
//         for res in results {
//             let m = &res.metrics;
//             println!("\nDesign: {}", m.design_name);
//             println!("  Gates:        {}", m.gate_count);
//             println!("  Peak RSS:     {:.2}MB", m.peak_rss_kb as f64 / 1024.0);
//             println!("  Query Times:");
//             for (name, time) in &m.query_times_ms {
//                 println!("    {:<15} {}ms", name, time);
//             }
//         }

//         println!(
//             "\n========================================\nMATCH SUMMARY\n========================================"
//         );
//         println!(
//             "{:<20} | {:<30} | Matches\n{}",
//             "Query",
//             "Design",
//             "-".repeat(65)
//         );
//         for res in results {
//             for qc in &res.counts {
//                 println!("{:<20} | {:<30} | {}", qc.query, qc.design, qc.count);
//             }
//         }
//         Ok(())
//     }

//     fn render_json(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
//         let output: Vec<_> = results
//             .iter()
//             .map(|r| {
//                 serde_json::json!({
//                     "metrics": r.metrics,
//                     "findings": r.findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
//                 })
//             })
//             .collect();
//         let _lock = PRINT_LOCK.lock().unwrap();
//         println!("{}", serde_json::to_string_pretty(&output)?);
//         Ok(())
//     }

//     fn render_csv(&self, results: &[TaskResult]) -> Result<(), Box<dyn std::error::Error>> {
//         let _lock = PRINT_LOCK.lock().unwrap();
//         println!("query,design,instance_path,locations");
//         for res in results {
//             for f in &res.findings {
//                 let r = &f.summary;
//                 let mut locs: Vec<_> = r.locations.iter().collect();
//                 locs.sort_by_key(|l| &l.subquery);

//                 let flat_locs = locs
//                     .iter()
//                     .map(|l| {
//                         format!(
//                             "{}@{}:[{}]",
//                             l.subquery,
//                             l.file,
//                             format_line_ranges(&l.lines)
//                         )
//                     })
//                     .collect::<Vec<_>>()
//                     .join(" | ");

//                 println!(
//                     "{},{},{},\"{}\"",
//                     r.query, r.design, r.instance_path, flat_locs
//                 );
//             }
//         }

//         println!("\n# PERFORMANCE METRICS");
//         println!("design,gates,load_ms,peak_rss_kb,query,query_ms,matches");
//         for res in results {
//             let m = &res.metrics;
//             for qc in &res.counts {
//                 let q_time = m.query_times_ms.get(&qc.query).unwrap_or(&0);
//                 println!(
//                     "{},{},{},{},{},{}",
//                     m.design_name, m.gate_count, m.peak_rss_kb, qc.query, q_time, qc.count
//                 );
//             }
//         }
//         Ok(())
//     }
// }

// fn read_file_lines(path: &str) -> std::io::Result<Vec<String>> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     reader.lines().collect()
// }

// fn find_root(i: usize, parent: &mut [usize]) -> usize {
//     if parent[i] == i {
//         i
//     } else {
//         parent[i] = find_root(parent[i], parent);
//         parent[i]
//     }
// }

// fn union_sets(i: usize, j: usize, parent: &mut [usize]) {
//     let root_i = find_root(i, parent);
//     let root_j = find_root(j, parent);
//     if root_i != root_j {
//         parent[root_i] = root_j;
//     }
// }

// fn format_line_ranges(lines: &[usize]) -> String {
//     if lines.is_empty() {
//         return String::new();
//     }
//     let mut ranges = Vec::new();
//     let mut i = 0;
//     while i < lines.len() {
//         let start = lines[i];
//         let mut end = start;
//         while i + 1 < lines.len() && lines[i + 1] == end + 1 {
//             i += 1;
//             end = lines[i];
//         }
//         if start == end {
//             ranges.push(start.to_string());
//         } else {
//             ranges.push(format!("{}-{}", start, end));
//         }
//         i += 1;
//     }
//     ranges.join(",")
// }

// fn extract_summary(query: &str, design: &str, node: ReportNode) -> MatchSummary {
//     let mut locations = HashSet::new();
//     collect_locations(&node, &mut locations);
//     MatchSummary {
//         query: query.to_string(),
//         design: design.to_string(),
//         instance_path: node.path.inst_path(),
//         locations: locations.into_iter().collect(),
//     }
// }

// fn collect_locations(node: &ReportNode, set: &mut HashSet<Location>) {
//     if let Some(loc) = &node.source_loc
//         && !loc.lines.is_empty()
//     {
//         set.insert(Location {
//             subquery: node.name.clone(),
//             file: loc.file.to_string(),
//             lines: loc.lines.iter().map(|l| l.number).collect(),
//         });
//     }

//     node.children
//         .iter()
//         .for_each(|child| collect_locations(child, set));
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt()
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .init();

//     let args = AppArgs::parse();
//     let config: CollectionConfig = serde_json::from_reader(File::open(&args.config_path)?)?;

//     let collector = Collector::new()?;
//     let results = collector.run_tasks(&config.designs, &args.query_name);

//     let reporter = Reporter::new(args.format);
//     reporter.render(&results)?;

//     Ok(())
// }

fn main() {
    println!("wip");
}
