use std::any::TypeId;
use svql_common::PortDirection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortColumn {
    pub name: &'static str,
    pub direction: PortDirection,
    pub nullable: bool,
}

impl PortColumn {
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Input,
            nullable: false,
        }
    }
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Output,
            nullable: false,
        }
    }
    pub const fn inout(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Inout,
            nullable: false,
        }
    }
    pub const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireArrayColumn {
    pub name: &'static str,
    pub nullable: bool,
}

impl WireArrayColumn {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            nullable: false,
        }
    }
    pub const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubColumn {
    pub name: &'static str,
    pub target_type: TypeId,
    pub target_type_name: &'static str,
    pub nullable: bool,
}

impl SubColumn {
    pub fn of<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            target_type: TypeId::of::<T>(),
            target_type_name: std::any::type_name::<T>(),
            nullable: false,
        }
    }
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaColumn {
    pub name: &'static str,
}

impl MetaColumn {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaColumn {
    Port(PortColumn),
    WireArray(WireArrayColumn),
    Sub(SubColumn),
    Meta(MetaColumn),
}

impl SchemaColumn {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Port(c) => c.name,
            Self::WireArray(c) => c.name,
            Self::Sub(c) => c.name,
            Self::Meta(c) => c.name,
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            Self::Port(c) => c.nullable,
            Self::WireArray(c) => c.nullable,
            Self::Sub(c) => c.nullable,
            Self::Meta(_) => false,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Port(_) => "Port",
            Self::WireArray(_) => "WireArray",
            Self::Sub(_) => "Sub",
            Self::Meta(_) => "Meta",
        }
    }

    pub fn as_port(&self) -> Option<&PortColumn> {
        match self {
            Self::Port(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_sub(&self) -> Option<&SubColumn> {
        match self {
            Self::Sub(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_meta(&self) -> Option<&MetaColumn> {
        match self {
            Self::Meta(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_wire_array(&self) -> Option<&WireArrayColumn> {
        match self {
            Self::WireArray(c) => Some(c),
            _ => None,
        }
    }

    pub fn is_port(&self) -> bool {
        matches!(self, Self::Port(_))
    }
    pub fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }
    pub fn is_meta(&self) -> bool {
        matches!(self, Self::Meta(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColIdx(u32);

impl ColIdx {
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubIdx(u32);

impl SubIdx {
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}
