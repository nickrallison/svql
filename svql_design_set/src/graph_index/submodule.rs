use std::borrow::Cow;

use prjunnamed_netlist::{Cell, Design};

#[derive(Debug, Clone)]
pub(crate) struct SubmoduleContainer {
    submodules: Vec<SubmoduleKey>,
}

impl SubmoduleContainer {
    pub fn new() -> Self {
        Self {
            submodules: Vec::new(),
        }
    }

    pub fn add_submodule(&mut self, name: String) {
        self.submodules.push(SubmoduleKey::new(name));
    }

    pub fn get_submodules(&self) -> &Vec<SubmoduleKey> {
        &self.submodules
    }

    pub fn build(design: &Design) -> Self {
        let mut container = SubmoduleContainer::new();
        for cell_ref in design.iter_cells() {
            if let Cow::Borrowed(Cell::Other(inst)) = cell_ref.get() {
                container.add_submodule(inst.kind.to_string());
            }
        }
        container
    }
}

impl From<Vec<String>> for SubmoduleContainer {
    fn from(value: Vec<String>) -> Self {
        Self {
            submodules: value.into_iter().map(SubmoduleKey::new).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SubmoduleKey {
    name: String,
}

impl SubmoduleKey {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<String> for SubmoduleKey {
    fn from(value: String) -> Self {
        Self { name: value }
    }
}
