use crate::module::traits::RtlModuleResultTrait;
use std::sync::Arc;
use svql_common::mat::{SanitizedCellData, SanitizedQueryMatch};

#[derive(Debug, Clone)]
pub struct RtlModuleResult<ModuleResultType> {
    pub inst: Arc<String>,
    pub full_path: Vec<Arc<String>>,
    // ################
    pub cells: Vec<SanitizedCellData>,
    pub module: ModuleResultType,
}

impl<ModuleResultType> RtlModuleResult<ModuleResultType>
where
    ModuleResultType: RtlModuleResultTrait,
{
    fn new(
        inst: String,
        parent_path: Vec<Arc<String>>,
        cells: Vec<SanitizedCellData>,
        module: ModuleResultType,
    ) -> Self {
        let inst = Arc::new(inst);
        let mut full_path = parent_path;
        full_path.push(inst.clone());
        RtlModuleResult {
            inst,
            full_path,
            cells,
            module,
        }
    }
    pub(crate) fn from_match(m: SanitizedQueryMatch) -> Self {
        let module = ModuleResultType::from_portmap(m.port_map);
        Self {
            inst: Arc::new("".to_string()),
            full_path: vec![],
            cells: m.cell_map.into_values().collect(),
            module,
        }
    }
}
