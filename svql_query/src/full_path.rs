use std::{collections::VecDeque, sync::Arc};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FullPath {
    pub inst: Arc<String>,
    pub path: Vec<Arc<String>>,
    pub height: usize,
}

impl FullPath {

    // fn new(inst: Arc<String>, path: Vec<Arc<String>>, height: usize) -> Self {
    //     Self { inst, path, height }
    // }
    pub fn new_root(inst: String) -> Self {
        Self {
            inst: Arc::new(inst),
            path: Vec::new(),
            height: 0,
        }
    }
    pub fn get_item(&self, index: usize) -> Option<Arc<String>> {
        self.path.get(index).cloned()
    }
    pub fn create_child(&self, child: String) -> Self {
        let child = Arc::new(child);
        let mut new_path = self.path.clone();
        new_path.push(child.clone());
        Self { inst: child, path: new_path, height: self.height + 1 }
    }
    pub fn inst_path(&self) -> String {
        self.path
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(".")
    }

}