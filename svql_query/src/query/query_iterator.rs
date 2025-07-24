use crate::driver::DriverIterator;
use crate::query::result::RtlQueryResult;
use svql_common::mat::SanitizedQueryMatch;

pub struct RtlQueryQueryIterator<T> {
    pub(crate) matches:
        std::iter::Map<DriverIterator, fn(SanitizedQueryMatch) -> RtlQueryResult<T>>,
}

impl<T> Iterator for RtlQueryQueryIterator<T> {
    type Item = RtlQueryResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.matches.next()
    }
}
