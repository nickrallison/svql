use crate::query::traits::RtlQueryResultTrait;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RtlQueryResult<QueryResultType> {
    pub inst: Arc<String>,
    pub full_path: VecDeque<Arc<String>>,
    // ################
    pub query: QueryResultType,
}

impl<QueryResultType> RtlQueryResult<QueryResultType>
where
    QueryResultType: RtlQueryResultTrait,
{
    pub fn new(
        query: QueryResultType,
        inst: Arc<String>,
        parent_path: VecDeque<Arc<String>>,
    ) -> Self {
        let mut full_path = parent_path;
        full_path.push_back(inst.clone());
        RtlQueryResult {
            inst,
            full_path,
            query,
        }
    }
}
