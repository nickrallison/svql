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
    pub inst: Arc<String>,
    pub full_path: VecDeque<Arc<String>>,
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
            full_path: vec![].into(),
            // connections: QueryType::connect(&query),
            query,
        };
        query.init_full_path(vec![].into());
        query
    }

    #[allow(dead_code)]
    pub fn inst_path(&self) -> String {
        inst_path(&self.full_path)
    }

    // pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
    //     self.connections.insert(conn);
    // }

    pub(crate) fn init_full_path(&mut self, parent_path: VecDeque<Arc<String>>) {
        let mut full_path = parent_path.clone();
        full_path.push_back(self.inst.clone());
        self.full_path = full_path.clone();

        // Initialize full path for module's ports
        self.query.init_full_path(full_path);
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<RtlQueryResult<QueryType::Result>>, DriverError> {
        let inst = self.inst.clone();
        let full_path = self.full_path.clone();
        self.query.query(driver, inst, full_path)
    }
}
