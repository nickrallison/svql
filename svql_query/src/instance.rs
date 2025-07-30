use std::{collections::VecDeque, sync::Arc};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Instance {
    pub inst: Arc<String>,
    pub path: Vec<Arc<String>>,
    pub height: usize,
}

impl Instance {
    pub fn root(inst: String) -> Self {
        let inst = Arc::new(inst);
        let new = Self {
            inst: inst.clone(),
            path: Vec::new(),
            height: 0,
        };
        debug_assert!(new.get_item(new.height) == Some(inst));
        new
    }
    pub fn child(&self, child: String) -> Self {
        let child = Arc::new(child);
        let mut new_path = self.path.clone();
        new_path.push(child.clone());
        let new = Self { inst: child.clone(), path: new_path, height: self.height + 1 };
        debug_assert!(new.get_item(new.height) == Some(child));
        new
    }

    pub fn get_item(&self, index: usize) -> Option<Arc<String>> {
        self.path.get(index).cloned()
    }
    pub fn inst_path(&self) -> String {
        self.path
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(".")
    }

}