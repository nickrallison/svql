use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use svql_subgraph::cell::SourceLocation;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct ReportNode {
    pub name: String,
    pub type_name: String,
    pub path: Instance,
    pub details: Option<String>,
    pub source_loc: Option<SourceLocation>,
    pub children: Vec<ReportNode>,
}

impl ReportNode {
    pub fn render(&self) -> String {
        let mut cache = HashMap::new();
        self.render_with_cache(&mut cache)
    }

    pub fn render_with_cache(&self, cache: &mut HashMap<Arc<str>, Vec<String>>) -> String {
        let mut output = String::new();
        self.render_recursive(&mut output, "", true, true, cache);
        output
    }

    fn render_recursive(
        &self,
        f: &mut String,
        prefix: &str,
        is_last: bool,
        is_root: bool,
        cache: &mut HashMap<Arc<str>, Vec<String>>,
    ) {
        let marker = if is_root {
            ""
        } else if is_last {
            "+-- "
        } else {
            "|-- "
        };

        let type_info = if let Some(ref d) = self.details {
            format!("({}: {})", self.type_name, d)
        } else {
            format!("({})", self.type_name)
        };

        let source_header = if let Some(source_loc) = &self.source_loc {
            if !source_loc.lines.is_empty() {
                format!(": {}:", source_loc.file)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        f.push_str(&format!(
            "{}{}{} {}{}\n",
            prefix, marker, self.name, type_info, source_header
        ));

        let new_prefix = if is_root {
            ""
        } else if is_last {
            &format!("{}    ", prefix)
        } else {
            &format!("{}|   ", prefix)
        };

        // Fetch and print source lines
        if let Some(source_loc) = &self.source_loc {
            if !source_loc.lines.is_empty() {
                let file_path = &source_loc.file;
                let lines = cache
                    .entry(file_path.clone())
                    .or_insert_with(|| read_file_lines(file_path).unwrap_or_default());

                for line_meta in &source_loc.lines {
                    // SourceLine numbers are 1-indexed
                    let content = if line_meta.number > 0 && line_meta.number <= lines.len() {
                        lines[line_meta.number - 1].trim_end()
                    } else {
                        "<line not found in file>"
                    };

                    f.push_str(&format!(
                        "{}    {:>4} | {}\n",
                        new_prefix, line_meta.number, content
                    ));
                }
            }
        } else {
            // No source location available
            f.push_str(&format!("{}    <no source location>\n", new_prefix));
        }

        for (i, child) in self.children.iter().enumerate() {
            let last_child = i == self.children.len() - 1;
            child.render_recursive(f, new_prefix, last_child, false, cache);
        }
    }
}

fn read_file_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
