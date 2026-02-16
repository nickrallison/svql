//! Simple columnar storage for ColumnEntry data.
//!
//! This replaces the polars DataFrame with a simpler implementation
//! that's sufficient for our use case of storing ColumnEntry values.

use crate::prelude::*;
use contracts::*;

/// A simple columnar storage structure for ColumnEntry data.
///
/// Stores data in column-major order, where each column is a `Vec<ColumnEntry>`.
#[derive(Debug, Clone)]
pub struct ColumnStore {
    /// Column data: column name -> Vec of typed entries
    columns: HashMap<String, Vec<ColumnEntry>>,
    /// Number of rows in the store.
    num_rows: usize,
    /// Ordered list of column names.
    column_names: Vec<String>,
}

impl ColumnStore {
    /// Get a mutable reference to a column by name.
    pub fn column_mut(&mut self, name: &str) -> Option<&mut Vec<ColumnEntry>> {
        self.columns.get_mut(name)
    }
}

impl ColumnStore {
    /// Validates the internal state of the columnar store.
    ///
    /// Ensures all columns have the same length and match the expected
    /// row count of the store.
    fn is_valid_state(&self) -> bool {
        self.columns.values().all(|col| col.len() == self.num_rows)
            && self
                .column_names
                .iter()
                .all(|name| self.columns.contains_key(name))
    }

    /// Create an empty ColumnStore with the given column names.
    #[must_use]
    #[ensures(ret.is_valid_state())]
    pub fn new(column_names: Vec<String>) -> Self {
        let mut columns = HashMap::with_capacity(column_names.len());
        for name in &column_names {
            columns.insert(name.clone(), Vec::new());
        }
        Self {
            columns,
            num_rows: 0,
            column_names,
        }
    }

    /// Create a ColumnStore from column data.
    ///
    /// All columns must have the same length.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of column names does not match the data or
    /// if columns have inconsistent lengths.
    #[ensures(ret.as_ref().is_ok_and(|s| s.is_valid_state()))]
    pub fn from_columns(
        column_names: Vec<String>,
        data: Vec<Vec<ColumnEntry>>,
    ) -> Result<Self, String> {
        if column_names.len() != data.len() {
            return Err(format!(
                "Column count mismatch: {} names but {} data columns",
                column_names.len(),
                data.len()
            ));
        }

        let num_rows = data.first().map(Vec::len).unwrap_or(0);

        // Verify all columns have the same length
        for (i, col) in data.iter().enumerate() {
            if col.len() != num_rows {
                return Err(format!(
                    "Column '{}' has {} rows but expected {}",
                    column_names[i],
                    col.len(),
                    num_rows
                ));
            }
        }

        let mut columns = HashMap::with_capacity(column_names.len());
        for (name, col_data) in column_names.iter().zip(data.into_iter()) {
            columns.insert(name.clone(), col_data);
        }

        Ok(Self {
            columns,
            num_rows,
            column_names,
        })
    }

    /// Get the number of rows.
    #[inline]
    pub const fn height(&self) -> usize {
        self.num_rows
    }

    /// Get a column by name.
    pub fn column(&self, name: &str) -> Option<&Vec<ColumnEntry>> {
        self.columns.get(name)
    }

    /// Get the column names in order.
    pub fn column_names(&self) -> &[String] {
        &self.column_names
    }

    /// Appends a full row of entries to the store
    ///
    /// # Panics
    ///
    /// Panics if the column name does not exist in the store schema.
    #[ensures(self.is_valid_state())]
    pub fn push_row(&mut self, row: EntryArray) {
        for (name, entry) in self.column_names.iter().zip(row.entries.into_iter()) {
            self.columns.get_mut(name).unwrap().push(entry);
        }
        self.num_rows += 1;
    }

    /// Retrieves a single cell entry by column name and row index.
    pub fn get_cell(&self, col_name: &str, row_idx: usize) -> &ColumnEntry {
        &self.columns[col_name][row_idx]
    }

    /// Deduplicate rows based on all columns.
    ///
    /// Keeps the first occurrence of each unique row.
    ///
    /// # Panics
    ///
    /// Panics if the column name does not exist in the store schema.
    pub fn deduplicate(&self) -> Self {
        let mut seen = HashSet::<Vec<&ColumnEntry>>::new();
        let mut new_columns: HashMap<String, Vec<ColumnEntry>> = self
            .column_names
            .iter()
            .map(|name: &String| (name.clone(), Vec::<ColumnEntry>::new()))
            .collect();

        let mut new_row_count = 0;

        for row_idx in 0..self.num_rows {
            // Create a signature for this row
            let mut row_sig = Vec::with_capacity(self.column_names.len());
            for col_name in &self.column_names {
                if let Some(col) = self.columns.get(col_name) {
                    row_sig.push(&col[row_idx]);
                }
            }

            if seen.insert(row_sig) {
                // This row is unique, keep it
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        new_columns
                            .get_mut(col_name)
                            .unwrap()
                            .push(col[row_idx].clone());
                    }
                }
                new_row_count += 1;
            }
        }

        Self {
            columns: new_columns,
            num_rows: new_row_count,
            column_names: self.column_names.clone(),
        }
    }

    /// Deduplicate rows based on a subset of columns.
    ///
    /// Keeps the first occurrence of each unique row.
    ///
    /// # Panics
    ///
    /// Panics if the column name does not exist in the store schema.
    pub fn deduplicate_subset(&self, subset: &[String]) -> Self {
        let mut seen = HashSet::<Vec<&ColumnEntry>>::new();
        let mut new_columns: HashMap<String, Vec<ColumnEntry>> = self
            .column_names
            .iter()
            .map(|name: &String| (name.clone(), Vec::<ColumnEntry>::new()))
            .collect();

        let mut new_row_count = 0;

        for row_idx in 0..self.num_rows {
            // Create a signature for this row based on subset columns
            let mut row_sig = Vec::with_capacity(subset.len());
            for col_name in subset {
                if let Some(col) = self.columns.get(col_name) {
                    row_sig.push(&col[row_idx]);
                }
            }

            if seen.insert(row_sig) {
                // This row is unique based on subset, keep it
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        new_columns
                            .get_mut(col_name)
                            .unwrap()
                            .push(col[row_idx].clone());
                    }
                }
                new_row_count += 1;
            }
        }

        Self {
            columns: new_columns,
            num_rows: new_row_count,
            column_names: self.column_names.clone(),
        }
    }
}

impl std::fmt::Display for ColumnStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.column_names.is_empty() {
            return writeln!(f, "Empty table");
        }

        // Helper to format entries consistently for width calculation and printing
        let format_entry = |entry: &ColumnEntry| -> String {
            match entry {
                ColumnEntry::Null => "null".to_string(),
                ColumnEntry::Wire(wire_ref) => format!("{wire_ref}"),
                ColumnEntry::WireArray(wires) => format!("[{}]", wires.len()),
                ColumnEntry::Sub(idx) => idx.to_string(),
                ColumnEntry::Metadata(id) => id.to_string(),
            }
        };

        // 1. Precompute widths
        let mut col_widths: Vec<usize> =
            self.column_names.iter().map(|n: &String| n.len()).collect();
        let row_label_width = self.num_rows.to_string().len().max(3);

        // Scan all rows to find the maximum width per column
        for row_idx in 0..self.num_rows {
            for (i, col_name) in self.column_names.iter().enumerate() {
                let entry = &self.columns[col_name][row_idx];
                let entry_str = format_entry(entry);
                col_widths[i] = col_widths[i].max(entry_str.len());
            }
        }

        // 2. Draw Header
        write!(f, "│ {:>width$} │", "row", width = row_label_width)?;
        for (i, name) in self.column_names.iter().enumerate() {
            write!(f, " {:^width$} │", name, width = col_widths[i])?;
        }
        writeln!(f)?;

        // 3. Draw Separator
        write!(f, "├{:─^width$}┼", "─", width = row_label_width + 2)?;
        for (i, _) in self.column_names.iter().enumerate() {
            write!(f, "{:─^width$}┼", "─", width = col_widths[i] + 2)?;
        }
        writeln!(f)?;

        // 4. Draw Rows
        let print_row = |f: &mut std::fmt::Formatter<'_>, row_idx: usize| -> std::fmt::Result {
            write!(f, "│ {:>width$} │", row_idx, width = row_label_width)?;
            for (i, col_name) in self.column_names.iter().enumerate() {
                let entry = &self.columns[col_name][row_idx];
                let entry_str = format_entry(entry);
                write!(f, " {:>width$} │", entry_str, width = col_widths[i])?;
            }
            writeln!(f)
        };

        if self.num_rows <= 10 {
            for row_idx in 0..self.num_rows {
                print_row(f, row_idx)?;
            }
        } else {
            // First 5 rows
            for row_idx in 0..5 {
                print_row(f, row_idx)?;
            }

            // Ellipsis
            write!(f, "│ {:>width$} │", "...", width = row_label_width)?;
            for (i, _) in self.column_names.iter().enumerate() {
                write!(f, " {:>width$} │", "...", width = col_widths[i])?;
            }
            writeln!(f)?;
            writeln!(f, "... {} more rows", self.num_rows - 10)?;

            // Last 5 rows
            for row_idx in (self.num_rows - 5)..self.num_rows {
                print_row(f, row_idx)?;
            }
        }

        Ok(())
    }
}
