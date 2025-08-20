use std::collections::HashMap;

use svql_driver::prelude::{DesignKey, Driver};
use svql_driver::query_ctx::QueryCtx;

use crate::netlist::NetlistMeta;

/// A haystack-scoped pool of child QueryCtx instances.
/// - Owns the shared Driver and haystack key
/// - Lazily populates QueryCtx per pattern via ensure<M>()
/// - Returns &'ctx QueryCtx via get<M>(), borrowing from &self so results can
///   safely borrow the underlying Designs for as long as the pool is alive.
#[derive(Clone, Debug)]
pub struct HaystackPool {
    driver: Driver,
    hay_key: DesignKey,
    // No interior mutability in get(); ensure<M>() requires &mut self.
    map: HashMap<(&'static str, &'static str), QueryCtx>,
}

impl HaystackPool {
    pub fn new(driver: Driver, hay_key: DesignKey) -> Self {
        Self {
            driver,
            hay_key,
            map: HashMap::new(),
        }
    }

    /// Ensure the QueryCtx for a given netlist meta exists (insert on miss).
    pub fn ensure<M: NetlistMeta>(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let key = (M::FILE_PATH, M::MODULE_NAME);
        if !self.map.contains_key(&key) {
            let ctx = M::open_ctx(&self.driver, &self.hay_key)?;
            self.map.insert(key, ctx);
        }
        Ok(())
    }

    /// Borrow the QueryCtx for a given netlist meta. Panic if not ensured.
    pub fn get<M: NetlistMeta>(&self) -> &QueryCtx {
        let key = (M::FILE_PATH, M::MODULE_NAME);
        self.map
            .get(&key)
            .expect("HaystackPool::get<M> called before ensure<M>()")
    }

    pub fn driver(&self) -> &Driver {
        &self.driver
    }

    pub fn hay_key(&self) -> &DesignKey {
        &self.hay_key
    }
}
