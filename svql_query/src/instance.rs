use std::{sync::Arc};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Instance {
    pub inst: Arc<String>,
    pub path: Vec<Arc<String>>,
    pub height: usize,
}

impl Instance {
    pub fn root(inst: String) -> Self {
        let inst = Arc::new(inst);
        let path = vec![inst.clone()];
        let new = Self {
            inst: inst.clone(),
            path,
            height: 0,
        };

        // Debug assertions
        let actual = new.get_item(new.height);
        let expected = Some(inst.clone());
        debug_assert!(actual == expected, "Expected {:?}, but got {:?}", expected, actual);
        new
    }
    pub fn child(&self, child: String) -> Self {
        let child = Arc::new(child);
        let mut new_path = self.path.clone();
        new_path.push(child.clone());
        let new = Self { inst: child.clone(), path: new_path, height: self.height + 1 };

        // Debug assertions
        let actual = new.get_item(new.height);
        let expected = Some(child.clone());
        debug_assert!(actual == expected, "Expected {:?}, but got {:?}", expected, actual);
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
    pub fn height(&self) -> usize {
        self.height
    }

}