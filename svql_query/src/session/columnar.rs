//! Simple columnar storage for u32 data.
//!
//! This replaces the polars DataFrame with a simpler implementation
//! that's sufficient for our use case of storing u32 values.

use std::collections::HashMap;

/// A simple columnar storage structure for u32 data.
///
/// Stores data in column-major order, where each column is a Vec<Option<u32>>.
#[derive(Debug, Clone)]
pub struct ColumnStore {
    /// Column data: column name -> Vec<Option<u32>>
    columns: HashMap<String, Vec<Option<u32>>>,
    /// Number of rows in the table
    num_rows: usize,
    /// Column names in order (for iteration)
    column_names: Vec<String>,
}

impl ColumnStore {
    /// Create an empty ColumnStore with the given column names.
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
    pub fn from_columns(
        column_names: Vec<String>,
        data: Vec<Vec<Option<u32>>>,
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
    pub fn height(&self) -> usize {
        self.num_rows
    }

    /// Get a column by name.
    pub fn column(&self, name: &str) -> Option<&Vec<Option<u32>>> {
        self.columns.get(name)
    }

    /// Get a mutable reference to a column by name.
    pub fn column_mut(&mut self, name: &str) -> Option<&mut Vec<Option<u32>>> {
        self.columns.get_mut(name)
    }

    /// Get the column names in order.
    pub fn column_names(&self) -> &[String] {
        &self.column_names
    }

    /// Deduplicate rows based on all columns.
    ///
    /// Keeps the first occurrence of each unique row.
    pub fn deduplicate(&self) -> Self {
        let mut seen = std::collections::HashSet::new();
        let mut new_columns: HashMap<String, Vec<Option<u32>>> = self
            .column_names
            .iter()
            .map(|name| (name.clone(), Vec::new()))
            .collect();

        let mut new_row_count = 0;

        for row_idx in 0..self.num_rows {
            // Create a signature for this row
            let mut row_sig = Vec::with_capacity(self.column_names.len());
            for col_name in &self.column_names {
                if let Some(col) = self.columns.get(col_name) {
                    row_sig.push(col[row_idx]);
                }
            }

            if seen.insert(row_sig) {
                // This row is unique, keep it
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        new_columns.get_mut(col_name).unwrap().push(col[row_idx]);
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
    pub fn deduplicate_subset(&self, subset: &[String]) -> Self {
        let mut seen = std::collections::HashSet::new();
        let mut new_columns: HashMap<String, Vec<Option<u32>>> = self
            .column_names
            .iter()
            .map(|name| (name.clone(), Vec::new()))
            .collect();

        let mut new_row_count = 0;

        for row_idx in 0..self.num_rows {
            // Create a signature for this row based on subset columns
            let mut row_sig = Vec::with_capacity(subset.len());
            for col_name in subset {
                if let Some(col) = self.columns.get(col_name) {
                    row_sig.push(col[row_idx]);
                }
            }

            if seen.insert(row_sig) {
                // This row is unique based on subset, keep it
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        new_columns.get_mut(col_name).unwrap().push(col[row_idx]);
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

        // Write header
        write!(f, "│ row │")?;
        for name in &self.column_names {
            write!(f, " {name:>8} │")?;
        }
        writeln!(f)?;

        // Write separator
        write!(f, "├─────┤")?;
        for _ in &self.column_names {
            write!(f, "──────────┤")?;
        }
        writeln!(f)?;

        // Write data rows - show first 5 and last 5 if more than 10 rows
        if self.num_rows <= 10 {
            // Show all rows if 10 or fewer
            for row_idx in 0..self.num_rows {
                write!(f, "│ {row_idx:>3} │")?;
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        match col[row_idx] {
                            Some(val) => write!(f, " {val:>8} │")?,
                            None => write!(f, "     null │")?,
                        }
                    }
                }
                writeln!(f)?;
            }
        } else {
            // Show first 5 rows
            for row_idx in 0..5 {
                write!(f, "│ {row_idx:>3} │")?;
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        match col[row_idx] {
                            Some(val) => write!(f, " {val:>8} │")?,
                            None => write!(f, "     null │")?,
                        }
                    }
                }
                writeln!(f)?;
            }

            // Show ellipsis
            write!(f, "│ ... │")?;
            for _ in &self.column_names {
                write!(f, "      ... │")?;
            }
            writeln!(f)?;
            writeln!(f, "... {} more rows", self.num_rows - 10)?;

            // Show last 5 rows
            for row_idx in (self.num_rows - 5)..self.num_rows {
                write!(f, "│ {row_idx:>3} │")?;
                for col_name in &self.column_names {
                    if let Some(col) = self.columns.get(col_name) {
                        match col[row_idx] {
                            Some(val) => write!(f, " {val:>8} │")?,
                            None => write!(f, "     null │")?,
                        }
                    }
                }
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
