use crate::driver::DriverIterator;
use crate::module::result::RtlModuleResult;
use svql_common::mat::SanitizedQueryMatch;

#[derive(Debug, Clone)]
pub struct RtlModuleQueryIterator<T> {
    pub(crate) matches:
        std::iter::Map<DriverIterator, fn(SanitizedQueryMatch) -> RtlModuleResult<T>>,
}

impl<T> Iterator for RtlModuleQueryIterator<T> {
    type Item = RtlModuleResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.matches.next()
    }
}
