//! Security pattern implementations (CWE vulnerabilities).

/// Detects bypass of locks via internal modes.
pub mod cwe1234;
/// Detects uninitialized values on reset.
pub mod cwe1271;
/// Detects access control checks occurring after asset access.
pub mod cwe1280;
/// Reusable security-focused hardware building blocks.
pub mod primitives;

pub use cwe1234::*;
pub use cwe1271::*;
pub use cwe1280::*;
pub use primitives::*;
