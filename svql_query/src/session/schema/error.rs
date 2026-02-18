/// Errors produced by schema validation.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Unknown column '{0}'")]
    UnknownColumn(String),

    #[error("Column '{name}' has wrong kind: expected {expected}, got {actual}")]
    WrongKind {
        name: String,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("Sub column '{column}' expects type '{expected}' but got '{actual}'")]
    SubTypeMismatch {
        column: String,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("Required column '{column}' (index {index}) was not set")]
    MissingRequired { column: String, index: usize },

    #[error("Duplicate column name '{0}'")]
    DuplicateName(String),
}
