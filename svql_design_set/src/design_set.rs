use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use prjunnamed_netlist::Design;
use svql_common::{ModuleConfig, YosysModule};

use crate::design_container::DesignContainer;

#[derive(Clone, Debug)]
pub struct DesignSet {
    top_module: String,
    modules: BTreeMap<String, Arc<DesignContainer>>,
}

impl DesignSet {
    pub fn new(top_module: String, modules: BTreeMap<String, Arc<DesignContainer>>) -> Self {
        Self {
            top_module,
            modules,
        }
    }

    fn from_designs(top_module: String, designs: BTreeMap<String, Design>) -> Self {
        let modules = designs
            .into_iter()
            .map(|(name, design)| (name, Arc::new(DesignContainer::build(design))))
            .collect();

        Self::new(top_module, modules)
    }

    pub fn import_design(
        module_name: String,
        module_path: PathBuf,
        module_config: &ModuleConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys_module: YosysModule = YosysModule::new(
            module_path
                .to_str()
                .expect("Failed to convert module path to string"),
            &module_name,
        )?;
        let (top_module, modules) = yosys_module.import_design(module_config)?;

        Ok(Self::from_designs(top_module, modules))
    }

    pub fn import_design_yosys(
        module_name: String,
        module_path: PathBuf,
        module_config: &ModuleConfig,
        yosys: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys_module: YosysModule = YosysModule::new(
            module_path
                .to_str()
                .expect("Failed to convert module path to string"),
            &module_name,
        )?;
        let (top_module, modules) = yosys_module.import_design_yosys(module_config, yosys)?;

        Ok(Self::from_designs(top_module, modules))
    }

    pub fn import_design_raw(
        module_name: String,
        module_path: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let yosys_module: YosysModule = YosysModule::new(
            module_path
                .to_str()
                .expect("Failed to convert module path to string"),
            &module_name,
        )?;
        let (top_module, modules) = yosys_module.import_design_raw()?;

        Ok(Self::from_designs(top_module, modules))
    }

    pub fn top_module(&self) -> &str {
        &self.top_module
    }

    pub fn modules(&self) -> &BTreeMap<String, Arc<DesignContainer>> {
        &self.modules
    }

    pub fn get(&self, name: &str) -> Option<&Design> {
        self.modules.get(name).map(|container| container.design())
    }
}
