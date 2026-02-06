//! Display functionality for pattern matches
//!
//! Provides hierarchical tree display with source location information.

use crate::prelude::*;
use ahash::AHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

/// A node in a hierarchical match report
#[derive(Debug, Clone)]
pub struct ReportNode {
    /// Display name for this node (field name or type name)
    pub name: String,
    /// Full type name or port direction
    pub type_name: String,
    /// Additional details (cell ID, variant name, etc.)
    pub details: Option<String>,
    /// Source code location if available
    pub source_loc: Option<SourceLocation>,
    /// Child nodes (submodules, fields, etc.)
    pub children: Vec<ReportNode>,
}

impl ReportNode {
    /// Render the report tree as a formatted string
    pub fn render(&self) -> String {
        let mut cache = AHashMap::new();
        self.render_with_cache(&mut cache)
    }

    /// Render with file content caching for efficiency
    pub fn render_with_cache(&self, cache: &mut AHashMap<Arc<str>, Vec<String>>) -> String {
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
        cache: &mut AHashMap<Arc<str>, Vec<String>>,
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

        // Print source lines if available
        if let Some(source_loc) = self.source_loc.as_ref().filter(|s| !s.lines.is_empty()) {
            let file_path = &source_loc.file;
            let lines = cache
                .entry(file_path.clone())
                .or_insert_with(|| read_file_lines(file_path).unwrap_or_default());

            for line_meta in &source_loc.lines {
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

/// Get source location for a wire if it references a cell
pub fn wire_source_location(
    wire: &Wire,
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> Option<SourceLocation> {
    let cell_id = wire.cell_id()?;
    let design = driver.get_design(key, &config.haystack_options).ok()?;
    let cell_wrapper = design.index().get_cell_by_id(cell_id.as_usize())?;
    cell_wrapper.get_source()
}

/// Create a report node for a wire field
pub fn wire_to_report_node(
    name: &str,
    wire: &Wire,
    direction: PortDirection,
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> ReportNode {
    let source_loc = wire_source_location(wire, driver, key, config);

    let details = match wire {
        Wire::Cell { id, .. } => Some(format!("cell_{}", id.raw())),
        Wire::PrimaryPort { name, .. } => Some(format!("port_{}", name)),
        Wire::Constant { value } => Some(format!("const_{}", value)),
    };

    ReportNode {
        name: name.to_string(),
        type_name: format!("{:?}", direction),
        details,
        source_loc,
        children: vec![],
    }
}
