//! Security pattern implementations (CWE vulnerabilities).

pub mod cwe1234;
pub mod cwe1271;
pub mod cwe1280;
pub mod primitives;

pub use cwe1234::*;
pub use cwe1271::*;
pub use cwe1280::*;
pub use primitives::*;
