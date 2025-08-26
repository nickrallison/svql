use std::sync::Arc;
use tracing::trace;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Instance {
    pub inst: Arc<str>,
    pub path: Vec<Arc<str>>,
    pub height: usize,
}

impl Instance {
    #[contracts::debug_ensures(ret.height == 0)]
    #[contracts::debug_ensures(ret.get_item(0) == Some(Arc::from(inst_in)))]
    pub fn root(inst_in: String) -> Self {
        let inst: Arc<str> = Arc::from(inst_in.clone());
        let path = vec![inst.clone()];
        trace!("Creating root instance: {}", inst_in);
        Self {
            inst: inst.clone(),
            path,
            height: 0,
        }
    }
    #[contracts::debug_ensures(ret.height == self.height + 1)]
    #[contracts::debug_ensures(ret.get_item(ret.height).is_some())]
    #[contracts::debug_ensures(ret.get_item(ret.height + 1).is_none())]
    pub fn child(&self, child: String) -> Self {
        let child: Arc<str> = Arc::from(child);
        let mut new_path = self.path.clone();
        new_path.push(child.clone());
        trace!("Creating child instance: {}", child);
        let new = Self {
            inst: child.clone(),
            path: new_path,
            height: self.height + 1,
        };

        // Debug assertions
        let actual = new.get_item(new.height);
        let expected = Some(child.clone());
        debug_assert!(
            actual == expected,
            "Expected {:?}, but got {:?}",
            expected,
            actual
        );
        new
    }

    pub fn get_item(&self, index: usize) -> Option<Arc<str>> {
        self.path.get(index).cloned()
    }
    #[contracts::debug_ensures(!ret.is_empty())]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;

    fn init_test_logger() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn instance_paths_and_heights() {
        init_test_logger();
        let r = Instance::root("root".to_string());
        assert_eq!(r.height(), 0);
        assert_eq!(r.inst_path(), "root");

        let c1 = r.child("a".to_string());
        assert_eq!(c1.height(), 1);
        assert_eq!(c1.inst_path(), "root.a");

        let c2 = c1.child("b".to_string());
        assert_eq!(c2.height(), 2);
        assert_eq!(c2.inst_path(), "root.a.b");

        assert_eq!(c2.get_item(0).unwrap().as_ref(), "root");
        assert_eq!(c2.get_item(1).unwrap().as_ref(), "a");
        assert_eq!(c2.get_item(2).unwrap().as_ref(), "b");
        assert!(c2.get_item(3).is_none());
    }
}
