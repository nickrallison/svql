/// Errors produced by schema validation.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// A column name was not found in the schema.
    #[error("Unknown column '{0}'")]
    UnknownColumn(String),

    /// A column has the wrong kind for the requested operation.
    #[error("Column '{name}' has wrong kind: expected {expected}, got {actual}")]
    WrongKind {
        /// The column name.
        name: String,
        /// The expected kind.
        expected: &'static str,
        /// The actual kind.
        actual: &'static str,
    },

    /// A sub-column's target type does not match the expected type.
    #[error("Sub column '{column}' expects type '{expected}' but got '{actual}'")]
    SubTypeMismatch {
        /// The column name.
        column: String,
        /// The expected type name.
        expected: &'static str,
        /// The actual type name.
        actual: &'static str,
    },

    /// A required column was not set.
    #[error("Required column '{column}' (index {index}) was not set")]
    MissingRequired {
        /// The column name.
        column: String,
        /// The column index.
        index: usize,
    },

    /// A column name appears more than once in the schema.
    #[error("Duplicate column name '{0}'")]
    DuplicateName(String),
}
