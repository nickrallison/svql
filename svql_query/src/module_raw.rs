// module_raw.rs  – completely object-safe
use svql_common::mat::SanitizedQueryMatch;
use std::path::PathBuf;

use crate::net::SvqlQueryError;

pub trait ModuleRaw: Send + Sync {
    fn file_path   (&self) -> PathBuf;
    fn module_name (&self) -> &'static str;

    /// always returns SanitizedQueryMatch → object-safe
    fn query_raw(
        &self,
        addr : &str,
    ) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError>;
}