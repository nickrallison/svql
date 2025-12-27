use svql_subgraph::cell::SourceLocation;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct ReportNode {
    pub name: String,
    pub type_name: String,
    pub path: Instance,
    pub details: Option<String>,
    pub source_loc: SourceLocation,
    pub children: Vec<ReportNode>,
}

impl ReportNode {
    pub fn render(&self) -> String {
        let mut output = String::new();
        self.render_recursive(&mut output, "", true, true);
        output
    }

    fn render_recursive(&self, f: &mut String, prefix: &str, is_last: bool, is_root: bool) {
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

        let source = if !self.source_loc.lines.is_empty() {
            format!(": {}:", self.source_loc.file)
        } else {
            "".to_string()
        };

        f.push_str(&format!(
            "{}{}{} {}{}\n",
            prefix, marker, self.name, type_info, source
        ));

        let new_prefix = if is_root {
            ""
        } else if is_last {
            &format!("{}    ", prefix)
        } else {
            &format!("{}|   ", prefix)
        };

        // Print source lines if available
        for line in &self.source_loc.lines {
            f.push_str(&format!(
                "{}    {} | <source code>\n",
                new_prefix, line.number
            ));
        }

        for (i, child) in self.children.iter().enumerate() {
            let last_child = i == self.children.len() - 1;
            child.render_recursive(f, new_prefix, last_child, false);
        }
    }
}
