use std::{collections::BTreeMap, sync::Arc};

use crate::design_container::DesignContainer;

#[derive(Clone, Debug)]
pub struct DesignSet {
    top_module: String,
    modules: BTreeMap<String, Arc<DesignContainer>>,
}
