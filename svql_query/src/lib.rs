use svql_subgraph::CellWrapper;

use crate::instance::Instance;

pub mod binding;
pub mod composite;
pub mod instance;
pub mod netlist;
pub mod queries;

// ########################
// Type State Tags
// ########################

// ------------  Compile–time state tags  ------------
pub trait State: Clone {}
pub trait QueryableState: State {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Search;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Match<'ctx> {
    pub pat_cell_ref: Option<CellWrapper<'ctx>>,
    pub design_cell_ref: Option<CellWrapper<'ctx>>,
}

impl State for Search {}
impl State for Match<'_> {}
impl QueryableState for Search {}

// ########################
// Helpers
// ########################

#[macro_export]
macro_rules! impl_find_port {
    ($ty:ident, $($field:ident),+) => {
        fn find_port(&self, p: &Instance) -> Option<&$crate::Wire<S>> {
            let idx  = self.path.height() + 1;
            match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
                $(Some(stringify!($field)) => self.$field.find_port(p),)+
                _ => None,
            }
        }
    };
}

// ########################
// Core Traits & Containers
// ########################

pub trait WithPath<S>: Sized
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>>;
    fn path(&self) -> Instance;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire<S>
where
    S: State,
{
    pub path: Instance,
    pub val: Option<S>,
}

impl<S> Wire<S>
where
    S: State,
{
    pub fn with_val(path: Instance, val: S) -> Self {
        Self {
            path,
            val: Some(val),
        }
    }

    pub fn new(path: Instance) -> Self {
        Self { path, val: None }
    }
}

impl<S> WithPath<S> for Wire<S>
where
    S: State,
{
    #[contracts::debug_ensures(p.height() <= self.path.height() + (p.height() - self.path.height()))]
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        if p.height() < self.path.height() {
            return None;
        }

        let item = p
            .get_item(p.height())
            .expect("WithPath::find_port(p): cannot find item");
        let self_name = self
            .path
            .get_item(self.path.height())
            .expect("WithPath::find_port(p): cannot find item");

        if item == self_name { Some(self) } else { None }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

// ########################
// Containers
// ########################

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection<S>
where
    S: State,
{
    pub from: Wire<S>,
    pub to: Wire<S>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ###############
    // Instance Tests
    // ###############
    #[test]
    fn test_instance() {
        let inst = Instance::root("test".to_string());
        let child1 = inst.child("child1".to_string());
        let child2 = child1.child("child2".to_string());
        assert_eq!(inst.inst_path(), "test");
        assert_eq!(child1.inst_path(), "test.child1");
        assert_eq!(child2.inst_path(), "test.child1.child2");
        assert_eq!(child2.get_item(0), Some(Arc::from("test".to_string())));
        assert_eq!(child2.get_item(1), Some(Arc::from("child1".to_string())));
        assert_eq!(child2.get_item(2), Some(Arc::from("child2".to_string())));
        assert_eq!(child2.get_item(3), None);
    }
}
