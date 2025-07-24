use crate::query::traits::RtlQueryResultTrait;
use svql_common::mat::SanitizedCellData;

#[derive(Debug, Clone)]
pub struct RtlQueryResult<QueryResultType> {
    pub cells: Vec<SanitizedCellData>,
    pub query: QueryResultType,
}

impl<QueryResultType> RtlQueryResult<QueryResultType>
where
    QueryResultType: RtlQueryResultTrait,
{
    pub fn new(cells: Vec<SanitizedCellData>, query: QueryResultType) -> Self {
        RtlQueryResult { cells, query }
    }
}
