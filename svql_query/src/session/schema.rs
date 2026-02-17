//! Column schema definitions for DataFrame-based pattern storage.
//!
//! These types define the schema for pattern result tables, including
//! wire references, submodule references, and metadata columns.

use std::any::TypeId;

use crate::prelude::*;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
    /// A wire bundle pointing into the design.
    Wire,
    /// A submodule reference (`Ref<T>`) pointing into another pattern table.
    /// For self-referential types (like `RecOr` trees), use `Sub(TypeId::of::<Self>())`.
    Sub(TypeId),
    /// Metadata column (e.g., depth, flags) - not a reference to other data.
    Metadata,
    /// A bundle of wire references (e.g., all leaf inputs to a logic cone).
    /// Used for set-based connectivity checking with `#[connect_any]`.
    WireArray,
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
        matches!(self, Self::Wire)
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

    /// Check if this is a wire array column.
    #[must_use]
    pub const fn is_wire_array(&self) -> bool {
        matches!(self, Self::WireArray)
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
    #[must_use]
    pub const fn metadata(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Metadata,
            nullable: false,
            direction: PortDirection::None,
        }
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

    /// Returns the target TypeId if this column represents a submodule.
    #[must_use]
    pub const fn as_submodule(&self) -> Option<TypeId> {
        match &self.kind {
            ColumnKind::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Description of a pattern port declaration.
#[derive(Debug, Clone, Copy)]
pub struct PortDecl {
    /// Name matching the netlist or alias.
    pub name: &'static str,
    /// Flow direction.
    pub direction: PortDirection,
}

impl PortDecl {
    /// Declares an input port.
    #[must_use]
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Input,
        }
    }

    /// Declares an output port.
    #[must_use]
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Output,
        }
    }

    /// Declares an inout port.
    #[must_use]
    pub const fn inout(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Inout,
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
    pub target: crate::selector::Selector<'static>,
    /// The direction of the port.
    pub direction: PortDirection,
}

impl Alias {
    /// Creates an input alias.
    #[must_use]
    pub const fn input(name: &'static str, target: crate::selector::Selector<'static>) -> Self {
        Self {
            port_name: name,
            target,
            direction: PortDirection::Input,
        }
    }

    /// Creates an output alias.
    #[must_use]
    pub const fn output(name: &'static str, target: crate::selector::Selector<'static>) -> Self {
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
    pub inner: crate::selector::Selector<'static>,
}

impl PortMap {
    /// Creates a new port mapping.
    #[must_use]
    pub const fn new(common: &'static str, inner: crate::selector::Selector<'static>) -> Self {
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
    /// A row index referencing another table.
    Sub(u32),
    /// Pattern metadata.
    Metadata(PhysicalCellId),
    /// A bundle of wires (e.g., list of leaf inputs).
    WireArray(Vec<Wire>),
}

impl ColumnEntry {
    /// Helper to extract u32 for backward compatibility/sorting
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Sub(idx) => Some(*idx),
            Self::Metadata(id) => Some(id.storage_key()),
            Self::Wire(wire) => wire.cell_id().map(|id| id.storage_key()),
            _ => None,
        }
    }

    /// Create a wire entry
    #[must_use]
    pub const fn wire(wire: Wire) -> Self {
        Self::Wire(wire)
    }

    /// Create a single net wire entry
    #[must_use]
    pub fn single_net(net_idx: u32) -> Self {
        let net = prjunnamed_netlist::Net::from_cell_index(net_idx as usize);
        Self::Wire(Wire::single(net))
    }

    /// Create a constant entry
    #[must_use]
    pub fn constant(value: bool) -> Self {
        let trit = prjunnamed_netlist::Trit::from(value);
        Self::Wire(Wire::constant(trit))
    }

    /// Create a submodule entry
    #[must_use]
    pub const fn sub(id: u32) -> Self {
        Self::Sub(id)
    }

    /// Create a metadata entry
    #[must_use]
    pub const fn metadata(id: PhysicalCellId) -> Self {
        Self::Metadata(id)
    }

    /// Create a wire array entry
    #[must_use]
    pub const fn wire_array(wires: Vec<Wire>) -> Self {
        Self::WireArray(wires)
    }
}

/// A fixed-size array of data representing one match result row.
#[derive(Debug, Clone)]
pub struct EntryArray {
    /// Ordered list of data corresponding to the pattern's schema.
    pub entries: Vec<ColumnEntry>,
}

impl EntryArray {
    /// Creates an empty array.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Pre-allocates an array with specific capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: vec![ColumnEntry::Null; capacity],
        }
    }
    /// Creates a new entry array from a vector of entries.
    pub(crate) const fn new(entries: Vec<ColumnEntry>) -> Self {
        Self { entries }
    }

    /// Generate a signature for deduplication that includes all cell IDs and submodule refs.
    ///
    /// Used for automatic deduplication - considers all wire references, submodule references,
    /// and metadata in the entry. Includes column index to distinguish entries with same values
    /// in different positions.
    #[must_use]
    pub fn signature(&self) -> Vec<(usize, u32)> {
        let mut sig: Vec<(usize, u32)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(col_idx, entry)| match entry {
                ColumnEntry::Wire(wire) => wire.cell_id().map(|id| (col_idx, id.storage_key())),
                ColumnEntry::WireArray(wires) => {
                    // Include all wires in the array in the signature
                    wires
                        .iter()
                        .filter_map(|w| w.cell_id().map(|id| (col_idx, id.storage_key())))
                        .next()
                }
                ColumnEntry::Sub(slot_idx) => Some((col_idx, *slot_idx)),
                ColumnEntry::Metadata(id) => Some((col_idx, id.storage_key())),
                _ => None,
            })
            .collect();
        sig.sort_unstable();
        sig
    }
}
