use crate::query::traits::RtlQueryResultTrait;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RtlQueryResult<QueryResultType> {
    #[allow(dead_code)]
    pub inst: Arc<String>,
    #[allow(dead_code)]
    pub instance: VecDeque<Arc<String>>,
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
        instance: VecDeque<Arc<String>>,
    ) -> Self {
        RtlQueryResult {
            inst,
            instance,
            query,
        }
    }
}
