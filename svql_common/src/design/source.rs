use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

pub fn read_file_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    BufReader::new(file).lines().collect::<Result<_, _>>()
}

/// Represents a physical location in the source code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    /// The originating source file.
    pub file: Arc<str>,
    /// The lines encompassing the hardware definition.
    pub lines: Vec<SourceLine>,
}

impl SourceLocation {
    /// Formats the source location for pretty-printed reports.
    #[must_use]
    pub fn report(&self) -> String {
        match self.lines.as_slice() {
            [] => format!("{}:<unknown>", self.file),
            [single] => format!("{}:{}", self.file, single.number),
            [first, .., last] => format!("{}:{}-{}", self.file, first.number, last.number),
        }
    }
}

/// Represents a specific line and column range within a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLine {
    /// 1-indexed line number.
    pub number: usize,
    /// Character offset where the definition starts.
    pub start_column: usize,
    /// Character offset where the definition ends.
    pub end_column: usize,
}

impl SourceLine {
    /// Formats the line and column range for reporting.
    #[must_use]
    pub fn report(&self) -> String {
        if self.end_column == 0 {
            format!("Line {}, Col {}+", self.number, self.start_column)
        } else {
            format!(
                "Line {}, Col {}-{}",
                self.number, self.start_column, self.end_column
            )
        }
    }
}
