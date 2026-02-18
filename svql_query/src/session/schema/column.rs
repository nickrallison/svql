use std::any::TypeId;
use svql_common::PortDirection;

/// A port column in a pattern schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortColumn {
    /// The name of this port.
    pub name: &'static str,
    /// The direction of this port.
    pub direction: PortDirection,
    /// Whether this port can be null.
    pub nullable: bool,
}

impl PortColumn {
    /// Create a new input port column.
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Input,
            nullable: false,
        }
    }
    /// Create a new output port column.
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Output,
            nullable: false,
        }
    }
    /// Create a new inout port column.
    pub const fn inout(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Inout,
            nullable: false,
        }
    }
    /// Mark this port as nullable.
    pub const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

/// A wire array column in a pattern schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireArrayColumn {
    /// The name of this wire array.
    pub name: &'static str,
    /// Whether this wire array can be null.
    pub nullable: bool,
}

impl WireArrayColumn {
    /// Create a new wire array column.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            nullable: false,
        }
    }
    /// Mark this wire array as nullable.
    pub const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

/// A sub-column referencing another pattern type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubColumn {
    /// The name of this sub-column.
    pub name: &'static str,
    /// The target pattern type ID.
    pub target_type: TypeId,
    /// The target pattern type name.
    pub target_type_name: &'static str,
    /// Whether this sub-column can be null.
    pub nullable: bool,
}

impl SubColumn {
    /// Create a new sub-column of type T.
    pub const fn of<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            target_type: TypeId::of::<T>(),
            target_type_name: std::any::type_name::<T>(),
            nullable: false,
        }
    }
    /// Mark this sub-column as nullable.
    pub const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

/// A metadata column in a pattern schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaColumn {
    /// The name of this metadata column.
    pub name: &'static str,
}

impl MetaColumn {
    /// Create a new metadata column.
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

/// A column in a pattern schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaColumn {
    /// A port column (wire with direction).
    Port(PortColumn),
    /// A wire array column.
    WireArray(WireArrayColumn),
    /// A sub-column referencing another pattern.
    Sub(SubColumn),
    /// A metadata column.
    Meta(MetaColumn),
}

impl SchemaColumn {
    /// Get the name of this column.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Port(c) => c.name,
            Self::WireArray(c) => c.name,
            Self::Sub(c) => c.name,
            Self::Meta(c) => c.name,
        }
    }

    /// Check if this column is nullable.
    pub const fn is_nullable(&self) -> bool {
        match self {
            Self::Port(c) => c.nullable,
            Self::WireArray(c) => c.nullable,
            Self::Sub(c) => c.nullable,
            Self::Meta(_) => false,
        }
    }

    /// Get the kind name of this column as a string.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Port(_) => "Port",
            Self::WireArray(_) => "WireArray",
            Self::Sub(_) => "Sub",
            Self::Meta(_) => "Meta",
        }
    }

    /// Extract the port column if this is a port.
    pub const fn as_port(&self) -> Option<&PortColumn> {
        match self {
            Self::Port(c) => Some(c),
            _ => None,
        }
    }

    /// Extract the sub-column if this is a sub-column.
    pub const fn as_sub(&self) -> Option<&SubColumn> {
        match self {
            Self::Sub(c) => Some(c),
            _ => None,
        }
    }

    /// Extract the meta column if this is a metadata column.
    pub const fn as_meta(&self) -> Option<&MetaColumn> {
        match self {
            Self::Meta(c) => Some(c),
            _ => None,
        }
    }

    /// Extract the wire array column if this is a wire array.
    pub const fn as_wire_array(&self) -> Option<&WireArrayColumn> {
        match self {
            Self::WireArray(c) => Some(c),
            _ => None,
        }
    }

    /// Check if this is a port column.
    pub const fn is_port(&self) -> bool {
        matches!(self, Self::Port(_))
    }
    /// Check if this is a sub-column.
    pub const fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }
    /// Check if this is a metadata column.
    pub const fn is_meta(&self) -> bool {
        matches!(self, Self::Meta(_))
    }
}

/// A column index in a pattern schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColIdx(u32);

impl ColIdx {
    /// Create a new column index.
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }
    /// Extract the raw column index.
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

/// A sub-column index in a pattern schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubIdx(u32);

impl SubIdx {
    /// Create a new sub-column index.
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }
    /// Extract the raw sub-column index.
    #[allow(dead_code)]
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}
