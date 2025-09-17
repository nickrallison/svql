use std::{collections::BTreeMap, sync::Arc};

use crate::graph_index::GraphIndex;
use ouroboros::self_referencing;
use prjunnamed_netlist::Design;
use svql_common::DesignSet;

pub struct DesignContainer {
    pub top_module: String,
    pub designs: BTreeMap<String, Arc<DesignInst>>,
}

impl DesignContainer {
    pub fn new(designs: DesignSet) -> Self {
        let top_module = designs.top_module;
        let designs_btree: BTreeMap<String, Design> = designs.modules;
        let designs = designs_btree
            .into_iter()
            .map(|(name, design)| (name, Arc::new(DesignInst::build(design))))
            .collect();

        Self {
            top_module,
            designs,
        }
    }

    pub fn get_design<'a>(&'a self, name: &str) -> Option<&'a Design> {
        self.designs
            .get(name)
            .map(|arc| arc.as_ref().borrow_design())
    }

    pub fn get_index<'a>(&'a self, name: &str) -> Option<&'a GraphIndex<'a>> {
        self.designs
            .get(name)
            .map(|arc| arc.as_ref().borrow_index())
    }
}

// #[derive(Debug)]
#[self_referencing]
pub struct DesignInst {
    design: Design,
    #[borrows(design)]
    #[covariant]
    index: GraphIndex<'this>,
}

impl std::fmt::Debug for DesignInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DesignInst")
            .field("design", &self.borrow_design())
            .field("index", &self.borrow_index())
            .finish()
    }
}

impl DesignInst {
    pub fn build(design: Design) -> Self {
        DesignInstBuilder {
            design,
            index_builder: |design: &Design| GraphIndex::build(design),
        }
        .build()
    }

    pub fn design(&self) -> &Design {
        self.borrow_design()
    }

    pub fn index(&self) -> &GraphIndex<'_> {
        self.borrow_index()
    }
}
