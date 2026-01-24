use crate::{
    prelude::ColumnDef,
    traits::{Component, kind},
};

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::SCHEMA.len();

    // the rest is tbd
}
