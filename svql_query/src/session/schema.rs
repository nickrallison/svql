//! Column schema definitions for DataFrame-based pattern storage.
//!
//! These types define the schema for pattern result tables, including
//! wire references, submodule references, and metadata columns.

use ahash::AHashMap;
use std::any::TypeId;
use std::sync::Arc;

use crate::prelude::*;
use crate::wire::WireRef;
// Assuming ahash is available, otherwise std HashMap

/// A smart wrapper around the raw column definitions.
///
/// This struct is initialized once per Pattern type and provides O(1) lookups
/// and pre-computed index lists to avoid repeated linear scans during execution.
#[derive(Debug)]
pub struct PatternSchema {
    /// The raw column definitions (source of truth).
    pub defs: &'static [ColumnDef],

    /// Fast lookup from column name to index.
    pub name_map: AHashMap<&'static str, usize>,

    /// Indices of all columns that are Submodules.
    pub submodules: Vec<usize>,

    /// Fast lookup from submodule column name to its compose target index.
    pub submodule_map: AHashMap<&'static str, usize>,

    /// Indices of all columns that are Inputs.
    pub inputs: Vec<usize>,

    /// Indices of all columns that are Outputs.
    pub outputs: Vec<usize>,
}

impl PatternSchema {
    /// Get a column definition by index.
    #[inline]
    #[must_use]
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
    pub fn new(defs: &'static [ColumnDef]) -> Self {
        let mut name_map = AHashMap::new();
        let mut submodules = Vec::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        let mut submodule_map = AHashMap::new();
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

    #[inline]
    #[must_use]
    pub fn submodule_index(&self, name: &str) -> Option<usize> {
        self.submodule_map.get(name).copied()
    }
}

/// The kind of data stored in a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
    /// A wire reference (`CellId`) pointing into the design.
    Cell,
    /// A submodule reference (Ref<T>) pointing into another pattern table.
    /// For self-referential types (like `RecOr` trees), use `Sub(TypeId::of::<Self>())`.
    Sub(TypeId),
    /// Metadata column (e.g., depth, flags) - not a reference to other data.
    Metadata,
}

impl ColumnKind {
    /// Create a `Sub` column kind for a specific pattern type.
    #[must_use]
    pub const fn sub<T: 'static>() -> Self {
        Self::Sub(TypeId::of::<T>())
    }

    /// Check if this is a wire column.
    #[must_use]
    pub const fn is_wire(&self) -> bool {
        matches!(self, Self::Cell)
    }

    /// Check if this is a submodule reference column.
    #[must_use]
    pub const fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }

    /// Check if this is a metadata column.
    #[must_use]
    pub const fn is_metadata(&self) -> bool {
        matches!(self, Self::Metadata)
    }

    /// Get the target `TypeId` for a Sub column, if applicable.
    #[must_use]
    pub const fn sub_type(&self) -> Option<TypeId> {
        match self {
            Self::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Defines the direction of a port column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Not a port (internal wire, submodule reference, or metadata).
    None,
    /// Input port (receives signal).
    Input,
    /// Output port (drives signal).
    Output,
    /// Bidirectional port.
    Inout,
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
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::Input,
        }
    }

    /// Create a wire column that acts as an Output port.
    #[must_use]
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::Output,
        }
    }

    /// Create a wire column (non-nullable by default).
    #[must_use]
    pub const fn wire(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a wire column that can be NULL.
    #[must_use]
    pub const fn wire_nullable(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a metadata column (non-nullable by default).
    #[must_use]
    pub const fn metadata(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Metadata,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a submodule reference column.
    /// Note: Cannot be `const` because `TypeId::of::<T>()` is not const-stable.
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

    #[must_use]
    pub const fn as_submodule(&self) -> Option<TypeId> {
        match &self.kind {
            ColumnKind::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Port declaration for netlist and variant schemas
#[derive(Debug, Clone, Copy)]
pub struct Port {
    pub name: &'static str,
    pub direction: PortDirection,
}

impl Port {
    #[must_use]
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Input,
        }
    }

    #[must_use]
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Output,
        }
    }

    #[must_use]
    pub const fn inout(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Inout,
        }
    }
}

/// Submodule declaration for composites
#[derive(Debug, Clone, Copy)]
pub struct Submodule {
    pub name: &'static str,
    pub type_id: TypeId,
}

impl Submodule {
    #[must_use]
    pub const fn of<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            type_id: TypeId::of::<T>(),
        }
    }
}

/// Alias: exposes a submodule's port as this composite's port
#[derive(Debug, Clone, Copy)]
pub struct Alias {
    pub port_name: &'static str,
    pub target: crate::selector::Selector<'static>,
    pub direction: PortDirection,
}

impl Alias {
    #[must_use]
    pub const fn input(name: &'static str, target: crate::selector::Selector<'static>) -> Self {
        Self {
            port_name: name,
            target,
            direction: PortDirection::Input,
        }
    }

    #[must_use]
    pub const fn output(name: &'static str, target: crate::selector::Selector<'static>) -> Self {
        Self {
            port_name: name,
            target,
            direction: PortDirection::Output,
        }
    }
}

/// Port mapping for variants
#[derive(Debug, Clone, Copy)]
pub struct PortMap {
    pub common_port: &'static str,
    pub inner: crate::selector::Selector<'static>,
}

impl PortMap {
    #[must_use]
    pub const fn new(common: &'static str, inner: crate::selector::Selector<'static>) -> Self {
        Self {
            common_port: common,
            inner,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColumnEntry {
    /// Wire reference (cell, primary port, or constant)
    Wire { value: Option<WireRef> },
    /// Submodule column: stores row index in the submodule's table
    Sub { id: Option<u32> },
    /// Metadata column: stores auxiliary data
    Metadata { id: Option<u32> },
}

impl ColumnEntry {
    #[must_use]
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Wire { value } => value.as_ref().and_then(WireRef::as_cell).map(CellId::raw),
            Self::Sub { id } => *id,
            Self::Metadata { id } => *id,
        }
    }

    /// Create a cell wire entry
    #[must_use]
    pub fn cell(id: Option<CellId>) -> Self {
        Self::Wire {
            value: id.map(WireRef::Cell),
        }
    }

    /// Create a primary port entry
    #[must_use]
    pub fn primary_port(name: Arc<str>) -> Self {
        Self::Wire {
            value: Some(WireRef::PrimaryPort(name)),
        }
    }

    /// Create a constant entry
    #[must_use]
    pub const fn constant(value: bool) -> Self {
        Self::Wire {
            value: Some(WireRef::Constant(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryArray {
    pub entries: Vec<ColumnEntry>,
}

impl EntryArray {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: vec![ColumnEntry::Metadata { id: None }; capacity],
        }
    }
    pub(crate) const fn new(entries: Vec<ColumnEntry>) -> Self {
        Self { entries }
    }

    /// Generate a signature for deduplication that includes all cell IDs and submodule refs.
    ///
    /// Used for `Dedupe::All` - considers all wire references, submodule references, and metadata in the entry.
    /// Includes column index to distinguish entries with same values in different positions.
    #[must_use]
    pub fn signature_all(&self) -> Vec<(usize, u32)> {
        let mut sig: Vec<(usize, u32)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(col_idx, entry)| match entry {
                ColumnEntry::Wire {
                    value: Some(wire_ref),
                } => wire_ref.as_cell().map(|cid| (col_idx, cid.raw())),
                ColumnEntry::Sub { id: Some(id) } => Some((col_idx, *id)),
                ColumnEntry::Metadata { id: Some(id) } => Some((col_idx, *id)),
                _ => None,
            })
            .collect();
        sig.sort_unstable();
        sig
    }

    /// Generate a signature for deduplication that only includes submodule references and metadata.
    ///
    /// Used for `Dedupe::Inner` - considers Sub entries and Metadata (e.g., variant discriminant), excluding external ports.
    /// Includes column index to distinguish entries with same values in different positions.
    #[must_use]
    pub fn signature_inner(&self) -> Vec<(usize, u32)> {
        let mut sig: Vec<(usize, u32)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(col_idx, entry)| match entry {
                ColumnEntry::Sub { id: Some(id) } => Some((col_idx, *id)),
                ColumnEntry::Metadata { id: Some(id) } => Some((col_idx, *id)),
                _ => None,
            })
            .collect();
        sig.sort_unstable();
        sig
    }

    /// Generate a signature for deduplication based on the specified strategy.
    ///
    /// This is the unified deduplication method that handles all strategies:
    /// - `Dedupe::None` - returns None (no deduplication should be performed)
    /// - `Dedupe::Inner` - considers only Sub and Metadata entries; if those produce an
    ///   empty signature (e.g., for pure Netlist types with only Wire entries), falls back to `All`
    /// - `Dedupe::All` - considers all entries (Wire, Sub, Metadata)
    #[must_use]
    pub fn signature(&self, dedupe: &svql_common::Dedupe) -> Option<Vec<(usize, u32)>> {
        match dedupe {
            svql_common::Dedupe::None => None,
            svql_common::Dedupe::Inner => {
                let inner_sig = self.signature_inner();
                // If inner signature is empty (no Sub/Metadata entries),
                // fall back to All to avoid collapsing distinct rows
                if inner_sig.is_empty() {
                    Some(self.signature_all())
                } else {
                    Some(inner_sig)
                }
            }
            svql_common::Dedupe::All => Some(self.signature_all()),
        }
    }
}
