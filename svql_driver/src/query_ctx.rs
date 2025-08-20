use std::sync::Arc;

use prjunnamed_netlist::Design;

/// A stable lifetime owner for borrowed subgraph matches.
/// Keep this alive for as long as you use the matches returned from queries.
#[derive(Clone, Debug)]
pub struct QueryCtx {
    pat: Arc<Design>,
    hay: Arc<Design>,
}

impl QueryCtx {
    pub fn new(pat: Arc<Design>, hay: Arc<Design>) -> Self {
        Self { pat, hay }
    }

    pub fn pat(&self) -> &Design {
        &self.pat
    }

    pub fn hay(&self) -> &Design {
        &self.hay
    }
}
