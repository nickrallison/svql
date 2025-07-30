use crate::driver::{Driver, DriverError};
use crate::module::inst_path;
use crate::query::result::RtlQueryResult;
use crate::query::traits::RtlQueryTrait;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;

pub mod result;
pub mod traits;

#[derive(Debug, Clone)]
pub struct RtlQuery<QueryType> {
    pub height: usize,
    pub inst: Arc<String>,
    pub instance: VecDeque<Arc<String>>,
    // ################
    // pub connections: HashSet<Connection<InPort, OutPort>>,
    pub query: QueryType,
}

impl<QueryType> RtlQuery<QueryType>
where
    QueryType: RtlQueryTrait + Debug,
{
    pub fn new(query: QueryType, inst: String) -> Self {
        let mut query = RtlQuery {
            inst: Arc::new(inst),
            instance: vec![].into(),
            // connections: QueryType::connect(&query),
            query,
            height: 0,
        };
        query.init_instance(vec![].into(), 0);
        query
    }

    #[allow(dead_code)]
    pub fn inst_path(&self) -> String {
        inst_path(&self.instance)
    }

    // pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
    //     self.connections.insert(conn);
    // }

    pub(crate) fn init_instance(&mut self, parent_path: VecDeque<Arc<String>>, height: usize) {
        let mut instance = parent_path.clone();
        instance.push_back(self.inst.clone());
        self.instance = instance.clone();
        self.height = height;

        // Initialize full path for module's ports
        self.query.init_instance(instance, height);
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<RtlQueryResult<QueryType::Result>>, DriverError> {
        let inst = self.inst.clone();
        let instance = self.instance.clone();
        self.query.query(driver, inst, instance, self.height)
    }
}
