//! Display functionality for pattern matches
//!
//! Provides hierarchical tree display with source location information.

use crate::prelude::*;

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
        let mut cache = HashMap::new();
        self.render_with_cache(&mut cache)
    }

    /// Render with file content caching for efficiency
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
    let physical_id = wire.cell_id()?;
    let design = driver.get_design(key, &config.haystack_options).ok()?;
    
    // Use Translation API to find the cell wrapper
    let node = design.index().resolve_node(physical_id)?;
    design.index().get_cell_by_index(node).get_source()
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
    match wire {
        Wire::Cell { id, .. } => {
            let design = driver.get_design(key, &config.haystack_options).ok();
            let cell_wrapper = design
                .as_ref()
                .and_then(|d| d.index().get_cell_by_id(id.raw() as usize));

            if let Some(cell) = cell_wrapper {
                let kind = cell.cell_type();
                if kind.is_input() {
                    let port_name = cell.input_name().unwrap_or("<unnamed>");
                    return ReportNode {
                        name: name.to_string(),
                        type_name: format!("{:?}", direction),
                        details: Some(format!("Port: {}", port_name)),
                        source_loc: None,
                        children: vec![],
                    };
                } else if kind.is_output() {
                    let port_name = cell.output_name().unwrap_or("<unnamed>");
                    return ReportNode {
                        name: name.to_string(),
                        type_name: format!("{:?}", direction),
                        details: Some(format!("Port: {}", port_name)),
                        source_loc: None,
                        children: vec![],
                    };
                }
                // Regular cell — try to get source
                let source_loc = cell.get_source();
                ReportNode {
                    name: name.to_string(),
                    type_name: format!("{:?}", direction),
                    details: Some(format!("CellId: {}", id.raw())),
                    source_loc,
                    children: vec![],
                }
            } else {
                // Cell not found in design — just show the raw id
                ReportNode {
                    name: name.to_string(),
                    type_name: format!("{:?}", direction),
                    details: Some(format!("CellId: {}", id.raw())),
                    source_loc: None,
                    children: vec![],
                }
            }
        }
        Wire::PrimaryPort {
            name: port_name,
            direction: port_dir,
        } => ReportNode {
            name: name.to_string(),
            type_name: format!("{:?}", direction),
            details: Some(format!("Port ({}): {}", port_dir, port_name)),
            source_loc: None,
            children: vec![],
        },
        Wire::Constant { value } => ReportNode {
            name: name.to_string(),
            type_name: format!("{:?}", direction),
            details: Some(format!("Const: {}", value)),
            source_loc: None,
            children: vec![],
        },
    }
}

/// Information about a wire in a match row.
///
/// Returned by `get_wire_info()` for programmatic access to wire details.
#[derive(Debug, Clone)]
pub struct WireInfo {
    /// The name of the wire field
    pub name: String,
    /// The wire reference (Cell, PrimaryPort, or Constant)
    pub wire: Wire,
    /// The direction from the schema
    pub direction: PortDirection,
    /// Source location if available
    pub source_loc: Option<SourceLocation>,
}

/// Get structured information about a wire without rendering.
///
/// Useful for custom formatting or analysis.
pub fn get_wire_info<T: Pattern + Component>(
    row: &Row<T>,
    wire_name: &str,
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> Option<WireInfo> {
    let wire = row.wire(wire_name)?;
    let direction = T::schema().get(wire_name)?.direction;
    let source_loc = wire_source_location(&wire, driver, key, config);

    Some(WireInfo {
        name: wire_name.to_string(),
        wire,
        direction,
        source_loc,
    })
}

/// Render a single wire field from a pattern match as a formatted report.
///
/// Uses the same tree structure as `render_row()` but focuses on one wire.
/// Shows direction, type, cell ID, and source code location.
pub fn render_wire<T: Pattern + Component>(
    row: &Row<T>,
    wire_name: &str,
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> Option<String> {
    let wire = row.wire(wire_name)?;
    let direction = T::schema().get(wire_name)?.direction;

    let node = wire_to_report_node(wire_name, &wire, direction, driver, key, config);

    Some(node.render())
}

/// Render a wire in compact single-line format for quick debugging.
///
/// Format: `wire_name (direction: type) @ file:line`
pub fn render_wire_compact<T: Pattern + Component>(
    row: &Row<T>,
    wire_name: &str,
    driver: &Driver,
    key: &DriverKey,
) -> Option<String> {
    let wire = row.wire(wire_name)?;
    let direction = T::schema().get(wire_name)?.direction;

    let config = Config::default();
    let source_loc = wire_source_location(&wire, driver, key, &config);

    let type_info = match &wire {
        Wire::Cell { id, .. } => format!("cell_{}", id.raw()),
        Wire::PrimaryPort { name, .. } => format!("port_{}", name),
        Wire::Constant { value } => format!("const_{}", value),
    };

    let location = source_loc
        .and_then(|loc| {
            loc.lines
                .first()
                .map(|l| format!("{}:{}", loc.file, l.number))
        })
        .unwrap_or_else(|| "<no source>".to_string());

    Some(format!(
        "{} ({:?}: {}) @ {}",
        wire_name, direction, type_info, location
    ))
}

/// Render just the source code lines for a wire, without tree formatting.
///
/// Useful for extracting source context without visual clutter.
pub fn render_wire_source_only<T: Pattern + Component>(
    row: &Row<T>,
    wire_name: &str,
    driver: &Driver,
    key: &DriverKey,
) -> Option<String> {
    let wire = row.wire(wire_name)?;
    let config = Config::default();
    let source_loc = wire_source_location(&wire, driver, key, &config)?;

    let mut output = String::new();
    output.push_str(&format!("{}:\n", source_loc.file));

    // Read file
    let lines = read_file_lines(&source_loc.file).ok()?;

    for line_meta in &source_loc.lines {
        if line_meta.number > 0 && line_meta.number <= lines.len() {
            let content = lines[line_meta.number - 1].trim_end();
            output.push_str(&format!("  {:>5} | {}\n", line_meta.number, content));
        }
    }

    Some(output)
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Netlist)]
    #[netlist(
        file = "examples/fixtures/basic/and/verilog/and_gate.v",
        module = "and_gate"
    )]
    pub struct TestAndGate {
        #[port(input)]
        pub a: Wire,
        #[port(input)]
        pub b: Wire,
        #[port(output)]
        pub y: Wire,
    }

    #[test]
    fn test_wire_reporting() -> Result<(), Box<dyn std::error::Error>> {
        let driver = Driver::new_workspace()?;
        let key = DriverKey::new(
            "examples/fixtures/basic/and/json/mixed_and_tree.json",
            "mixed_and_tree",
        );
        let config = Config::default();

        let store = crate::run_query::<TestAndGate>(&driver, &key, &config)?;
        let table = store.get::<TestAndGate>().ok_or("Table not found")?;
        let row = table.rows().next().ok_or("No rows found")?;

        // Test render_wire
        let report = render_wire(&row, "y", &driver, &key, &config).ok_or("render_wire failed")?;
        assert!(report.contains("y"));
        assert!(report.contains("(Output"));

        // Test render_wire_compact
        let compact =
            render_wire_compact(&row, "y", &driver, &key).ok_or("render_wire_compact failed")?;
        assert!(compact.contains("y"));
        assert!(compact.contains("Output"));

        // Test render_wire_source_only
        // Note: Source location might be missing in some JSON-loaded designs
        let _ = render_wire_source_only(&row, "y", &driver, &key);

        // Test get_wire_info
        let info =
            get_wire_info(&row, "y", &driver, &key, &config).ok_or("get_wire_info failed")?;
        assert_eq!(info.name, "y");
        assert_eq!(info.direction, PortDirection::Output);

        Ok(())
    }
}
