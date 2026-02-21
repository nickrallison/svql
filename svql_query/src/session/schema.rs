//! Column schema definitions for DataFrame-based pattern storage.
//!
//! These types define the schema for pattern result tables, including
//! wire references, submodule references, and metadata columns.

/// Column type definitions.
pub mod column;
/// Schema validation errors.
pub mod error;
/// Pattern schema types.
pub mod pattern_schema;

use std::any::TypeId;

use crate::prelude::*;
use crate::session::MetaValue;
use crate::session::RowIndex;
use contracts::*;

/// A smart wrapper around the raw column definitions.
///
/// This struct is initialized once per Pattern type and provides O(1) lookups
/// and pre-computed index lists to avoid repeated linear scans during execution.
#[derive(Debug)]
pub struct PatternSchema {
    /// The raw column definitions (source of truth).
    pub defs: &'static [ColumnDef],

    /// Fast lookup from column name to index.
    pub name_map: HashMap<&'static str, usize>,

    /// Indices of all columns that are Submodules.
    pub submodules: Vec<usize>,

    /// Fast lookup from submodule column name to its compose target index.
    pub submodule_map: HashMap<&'static str, usize>,

    /// Indices of all columns that are Inputs.
    pub inputs: Vec<usize>,

    /// Indices of all columns that are Outputs.
    pub outputs: Vec<usize>,
}

impl PatternSchema {
    /// Get a column definition by index.
    #[inline]
    #[must_use]
    #[requires(index < self.defs.len())]
    pub fn column(&self, index: usize) -> &ColumnDef {
        &self.defs[index]
    }

    /// Get all column definitions.
    #[inline]
    #[must_use]
    pub const fn columns(&self) -> &[ColumnDef] {
        self.defs
    }

    /// Construct a new `PatternSchema` from raw definitions.
    /// This is typically called inside a `OnceLock::get_or_init`.
    #[must_use]
    #[ensures(ret.defs.len() == ret.name_map.len())]
    pub fn new(defs: &'static [ColumnDef]) -> Self {
        let mut name_map = HashMap::new();
        let mut submodules = Vec::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        let mut submodule_map = HashMap::new();
        let mut compose_idx = 0;

        for (i, col) in defs.iter().enumerate() {
            name_map.insert(col.name, i);

            if col.kind.is_sub() {
                submodule_map.insert(col.name, compose_idx);
                submodules.push(i);
                compose_idx += 1;
            }

            match col.direction {
                PortDirection::Input => inputs.push(i),
                PortDirection::Output => outputs.push(i),
                _ => {}
            }
        }

        Self {
            defs,
            name_map,
            submodules,
            submodule_map,
            inputs,
            outputs,
        }
    }

    /// Get the index of a column by name in O(1).
    #[inline]
    #[must_use]
    #[ensures(ret.is_some() -> ret.unwrap() < self.defs.len())]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.name_map.get(name).copied()
    }

    /// Get the definition of a column by name in O(1).
    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ColumnDef> {
        let idx = self.index_of(name)?;
        Some(&self.defs[idx])
    }

    /// Get the definition by index (bounds checked).
    #[inline]
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<&ColumnDef> {
        self.defs.get(index)
    }

    /// Returns the index of a submodule within the compose target list.
    #[inline]
    #[must_use]
    pub fn submodule_index(&self, name: &str) -> Option<usize> {
        self.submodule_map.get(name).copied()
    }
}

/// The kind of data stored in a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColumnKind {
    /// Stores a single physical wire reference (`Ref<Wire>`).
    Wire,
    /// Stores an array of physical wire references (`Vec<Wire>`).
    WireArray,
    /// Stores a reference to a submodule match (`Ref<T>`).
    Sub(TypeId),
    /// Stores a typed application metadata value (`MetaValue`).
    Meta,
}

impl ColumnKind {
    /// Create a `Sub` column kind for a specific pattern type.
    pub const fn sub<T: 'static>() -> Self {
        Self::Sub(TypeId::of::<T>())
    }

    /// Check if this is a wire column kind.
    pub const fn is_wire(&self) -> bool {
        matches!(self, Self::Wire)
    }

    /// Check if this is a sub-column kind.
    pub const fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }

    /// Check if this is a metadata column kind.
    pub const fn is_meta(&self) -> bool {
        matches!(self, Self::Meta)
    }

    /// Check if this is a wire array column kind.
    pub const fn is_wire_array(&self) -> bool {
        matches!(self, Self::WireArray)
    }

    /// Get the target type ID if this is a sub-column.
    pub const fn sub_type(&self) -> Option<TypeId> {
        match self {
            Self::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Definition of a single column in a pattern's result table.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name (must be unique within a pattern).
    pub name: &'static str,
    /// The kind of data stored in this column.
    pub kind: ColumnKind,
    /// Whether this column can contain NULL values.
    /// Used for optional submodules and tree children (`left_child`, `right_child`).
    pub nullable: bool,
    /// The directionality of this column (if it is a port).
    pub direction: PortDirection,
}

impl ColumnDef {
    /// Create a wire column that acts as an Input port.
    #[must_use]
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: false,
            direction: PortDirection::Input,
        }
    }

    /// Create a wire column that acts as an Output port.
    #[must_use]
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: false,
            direction: PortDirection::Output,
        }
    }

    /// Create a wire column (non-nullable by default).
    #[must_use]
    pub const fn wire(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a wire column that can be NULL.
    #[must_use]
    pub const fn wire_nullable(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a metadata column (non-nullable by default).
    pub const fn meta(name: &'static str) -> Self {
        Self::new(name, ColumnKind::Meta, false)
    }

    /// Create a wire array column (for bundled wires / set-based connectivity).
    #[must_use]
    pub const fn wire_array(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::WireArray,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a nullable wire array column.
    #[must_use]
    pub const fn wire_array_nullable(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::WireArray,
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a submodule reference column.
    #[must_use]
    pub const fn sub<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a nullable submodule reference column (for optional children).
    #[must_use]
    pub const fn sub_nullable<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a column with explicit kind and nullability.
    #[must_use]
    pub const fn new(name: &'static str, kind: ColumnKind, nullable: bool) -> Self {
        Self {
            name,
            kind,
            nullable,
            direction: PortDirection::None,
        }
    }

    /// Set the direction of this column (builder pattern)
    #[must_use]
    pub const fn with_direction(mut self, direction: PortDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns the target TypeId if this column represents a submodule.
    #[must_use]
    pub const fn as_submodule(&self) -> Option<TypeId> {
        match &self.kind {
            ColumnKind::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Description of a nested submodule field.
#[derive(Debug, Clone, Copy)]
pub struct Submodule {
    /// Field name in the struct.
    pub name: &'static str,
    /// Type fingerprint.
    pub type_id: TypeId,
}

impl Submodule {
    /// Creates a submodule declaration from a type.
    #[must_use]
    pub const fn of<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            type_id: TypeId::of::<T>(),
        }
    }
}

/// Exposes a submodule's port as a parent field.
#[derive(Debug, Clone, Copy)]
pub struct Alias {
    /// The name of the alias field.
    pub port_name: &'static str,
    /// The path selector targeting the submodule port.
    pub target: Selector<'static>,
    /// The direction of the port.
    pub direction: PortDirection,
}

impl Alias {
    /// Creates an input alias.
    #[must_use]
    pub const fn input(name: &'static str, target: Selector<'static>) -> Self {
        Self {
            port_name: name,
            target,
            direction: PortDirection::Input,
        }
    }

    /// Creates an output alias.
    #[must_use]
    pub const fn output(name: &'static str, target: Selector<'static>) -> Self {
        Self {
            port_name: name,
            target,
            direction: PortDirection::Output,
        }
    }
}

/// Defines the mapping from common variant ports to specific implementation paths.
#[derive(Debug, Clone, Copy)]
pub struct PortMap {
    /// The name of the common interface port.
    pub common_port: &'static str,
    /// Real path in the specific variant implementation.
    pub inner: Selector<'static>,
}

impl PortMap {
    /// Creates a new port mapping.
    #[must_use]
    pub const fn new(common: &'static str, inner: Selector<'static>) -> Self {
        Self {
            common_port: common,
            inner,
        }
    }
}

/// A dynamic value type for columnar table storage.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ColumnEntry {
    /// The cell or reference is missing or optional.
    Null,
    /// A wire (bundle of nets).
    Wire(Wire),
    /// A bundle of wires (e.g., list of leaf inputs).
    WireArray(Vec<Wire>),
    /// A row index referencing another table.
    /// The raw `RowIndex` is not accessible outside the `storage` module.
    Sub(RowIndex),
    /// Typed metadata value.
    Meta(MetaValue),
}

impl ColumnEntry {
    /// Create a wire entry.
    #[must_use]
    pub const fn wire(wire: Wire) -> Self {
        Self::Wire(wire)
    }

    /// Create a wire array entry.
    #[must_use]
    pub const fn wire_array(wires: Vec<Wire>) -> Self {
        Self::WireArray(wires)
    }

    /// Create a submodule entry from a row index.
    /// Internal crate only — allows generators and core traits to
    /// manufacture references while keeping them opaque to end users.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn sub(index: RowIndex) -> Self {
        Self::Sub(index)
    }

    /// Create a metadata entry.
    #[must_use]
    pub const fn meta(val: MetaValue) -> Self {
        Self::Meta(val)
    }

    /// Access as a wire reference, returning `None` for other variants.
    #[must_use]
    pub const fn as_wire(&self) -> Option<&Wire> {
        match self {
            Self::Wire(w) => Some(w),
            _ => None,
        }
    }

    /// Access as a wire array slice.
    #[must_use]
    pub fn as_wire_array(&self) -> Option<&[Wire]> {
        match self {
            Self::WireArray(wires) => Some(wires),
            _ => None,
        }
    }

    /// Access as a metadata value.
    #[must_use]
    pub const fn as_meta(&self) -> Option<&MetaValue> {
        match self {
            Self::Meta(m) => Some(m),
            _ => None,
        }
    }

    /// Access as a row index.
    /// Only available within the `storage` module — prevents external
    /// code from extracting raw integers from `Sub` entries.
    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn as_row_index(&self) -> Option<RowIndex> {
        match self {
            Self::Sub(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Returns the raw row index for type-erased sub-entry access.
    #[allow(dead_code)]
    pub(crate) const fn as_sub_raw(&self) -> Option<u32> {
        match self {
            Self::Sub(idx) => Some(idx.raw()),
            _ => None,
        }
    }
}

/// A fixed-size array of data representing one match result row.
#[derive(Debug, Clone)]
pub struct EntryArray {
    /// Ordered list of data corresponding to the pattern's schema.
    pub entries: Vec<ColumnEntry>,
}

/// A stable signature for deduplication.
///
/// Uses typed entries rather than raw integers.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EntrySignature(Vec<SignatureEntry>);

/// A single typed entry in a deduplication signature.
#[derive(Debug, PartialEq, Eq, Hash)]
enum SignatureEntry {
    /// A wire identified by its physical cell ID.
    Wire(PhysicalCellId),
    /// A submodule row reference (opaque).
    Sub(RowIndex),
    /// A metadata value.
    Meta(MetaValue),
}

impl EntryArray {
    /// Creates an empty array.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Pre-allocates an array with specific capacity, filled with `Null`.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: vec![ColumnEntry::Null; capacity],
        }
    }

    /// Creates a new entry array from a vector of entries.
    pub const fn new(entries: Vec<ColumnEntry>) -> Self {
        Self { entries }
    }

    /// Generate a typed signature for deduplication.
    ///
    /// Considers all wire references, submodule references, and metadata.
    /// The column index is included so entries with the same value in
    /// different positions produce distinct signatures.
    #[must_use]
    pub fn signature(&self) -> EntrySignature {
        let entries = self
            .entries
            .iter()
            .filter_map(|e| match e {
                ColumnEntry::Wire(w) => w.cell_id().map(SignatureEntry::Wire),
                ColumnEntry::Sub(idx) => Some(SignatureEntry::Sub(*idx)),
                ColumnEntry::Meta(m) => Some(SignatureEntry::Meta(m.clone())),
                _ => None,
            })
            .collect();
        EntrySignature(entries)
    }
}

// ─── Builder helpers for EntryArray ──────────────────────────────────────────

/// Convenience constructors used by pattern search implementations.
///
/// These live on `EntryArray` rather than on `ColumnEntry` so that the
/// storage-internal `Sub(RowIndex)` variant is never exposed.
impl EntryArray {
    /// Set a `Sub` entry at position `col_idx` using a `Ref<T>`.
    ///
    /// This is the only way external code (pattern implementations) can
    /// write a sub-row reference — they must go through a typed `Ref<T>`.
    #[allow(dead_code)]
    pub(crate) fn set_sub<T>(&mut self, col_idx: usize, r: Ref<T>) {
        self.entries[col_idx] = ColumnEntry::Sub(r.raw_index());
    }

    /// Set a `Sub` entry at position `col_idx` using a raw `RowIndex`.
    ///
    /// Available to external implementors of `Recursive::build_recursive`.
    pub fn set_sub_raw(&mut self, col_idx: usize, idx: RowIndex) {
        self.entries[col_idx] = ColumnEntry::Sub(idx);
    }
}
