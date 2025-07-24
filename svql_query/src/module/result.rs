use crate::module::traits::RtlModuleResultTrait;
use svql_common::mat::{SanitizedCellData, SanitizedQueryMatch};

#[derive(Debug)]
pub struct RtlModuleResult<ModuleResultType> {
    // pub inst: String,
    pub cells: Vec<SanitizedCellData>,
    pub module: ModuleResultType,
}

impl<ModuleResultType> RtlModuleResult<ModuleResultType>
where
    ModuleResultType: RtlModuleResultTrait,
{
    fn new(cells: Vec<SanitizedCellData>, module: ModuleResultType) -> Self {
        RtlModuleResult { cells, module }
    }
    pub(crate) fn from_match(m: SanitizedQueryMatch) -> Self {
        let module = ModuleResultType::from_portmap(m.port_map);
        Self {
            cells: m.cell_map.into_values().collect(),
            module,
        }
    }
}
