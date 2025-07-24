use crate::query::traits::RtlQueryResultTrait;

#[derive(Debug, Clone)]
pub struct RtlQueryResult<QueryResultType> {
    // pub cells: Vec<SanitizedCellData>,
    pub query: QueryResultType,
}

impl<QueryResultType> RtlQueryResult<QueryResultType>
where
    QueryResultType: RtlQueryResultTrait,
{
    pub fn new(query: QueryResultType) -> Self {
        RtlQueryResult { query }
    }
}
