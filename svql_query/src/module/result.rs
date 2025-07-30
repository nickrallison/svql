use crate::module::traits::RtlModuleResultTrait;
use std::collections::VecDeque;
use std::sync::Arc;
use svql_common::matches::{SanitizedCellData, SanitizedQueryMatch};

#[derive(Debug, Clone)]
pub struct RtlModuleResult<ModuleResultType> {
    pub inst: Arc<String>,
    pub instance: VecDeque<Arc<String>>,
    // ################
    #[allow(dead_code)]
    pub cells: Vec<SanitizedCellData>,
    pub module: ModuleResultType,
}

impl<ModuleResultType> RtlModuleResult<ModuleResultType>
where
    ModuleResultType: RtlModuleResultTrait,
{
    pub(crate) fn from_match(m: SanitizedQueryMatch) -> Self {
        let module = ModuleResultType::from_portmap(m.port_map);
        Self {
            inst: Arc::new("".to_string()),
            instance: vec![].into(),
            cells: m.cell_map.into_values().collect(),
            module,
        }
    }
}
