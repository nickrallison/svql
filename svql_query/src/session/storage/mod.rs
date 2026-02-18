//! Columnar storage for query results.

/// Columnar storage implementation.
mod columnar;
/// Column entry types.
mod entry;
/// Entry array builder.
mod entry_array;
/// Metadata values.
pub mod meta_value;
/// Typed references to rows.
mod ref_type;
/// Row data type.
mod row;
/// Row index type.
mod row_index;
/// Store container.
mod store;
/// Table wrapper.
mod table;

pub use columnar::ColumnStore;
pub use entry::ColumnEntry;
pub use entry_array::{EntryArray, EntryArrayBuilder};
pub use meta_value::{MetaValue, VariantIndex};
pub use ref_type::Ref;
pub use row::Row;
pub use row_index::RowIndex;
pub use store::Store;
pub use table::{AnyTable, Table};
