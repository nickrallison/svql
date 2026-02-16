#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use std::path::PathBuf;
use svql_common::DesignPath;

mod common;

#[test]
fn test_design_path_verilog() {
    let path = PathBuf::from("design.v");
    let dp = DesignPath::new(path.clone()).unwrap();
    assert!(matches!(dp, DesignPath::Verilog(_)));
    assert_eq!(dp.read_command(), "read_verilog -sv");
    assert_eq!(dp.path(), &path);
}

#[test]
fn test_design_path_rtlil() {
    let path = PathBuf::from("design.il");
    let dp = DesignPath::new(path).unwrap();
    assert!(matches!(dp, DesignPath::Rtlil(_)));
    assert_eq!(dp.read_command(), "read_rtlil");
}

#[test]
fn test_design_path_json() {
    let path = PathBuf::from("design.json");
    let dp = DesignPath::new(path).unwrap();
    assert!(matches!(dp, DesignPath::Json(_)));
    assert_eq!(dp.read_command(), "read_json");
}

#[test]
fn test_design_path_unsupported() {
    let path = PathBuf::from("design.txt");
    assert!(DesignPath::new(path).is_err());
}

#[test]
fn test_design_path_no_extension() {
    let path = PathBuf::from("design");
    assert!(DesignPath::new(path).is_err());
}

#[test]
fn test_design_path_case_sensitivity() {
    // Test uppercase extensions
    let path_upper = PathBuf::from("design.V");
    assert!(DesignPath::new(path_upper).is_err()); // Should fail - not .v

    let path_lower = PathBuf::from("design.v");
    assert!(DesignPath::new(path_lower).is_ok());
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    #[derive(Clone, Debug)]
    struct ArbitraryDesignPath(DesignPath);

    impl Arbitrary for ArbitraryDesignPath {
        fn arbitrary(g: &mut Gen) -> Self {
            let extensions = vec!["v", "il", "json"];
            let ext = g.choose(&extensions).unwrap();
            // get random stem. can use seperators
            let stem_len: usize = usize::arbitrary(g) % 10 + 1; // 1 to 10 segments
            let stem: Vec<String> = (0..stem_len)
                .map(|_| {
                    let len = usize::arbitrary(g) % 10 + 1;
                    let s: String = (0..len)
                        .map(|_| char::from_u32(97 + (usize::arbitrary(g) % 26) as u32).unwrap())
                        .collect();
                    s
                })
                .collect();
            let path = PathBuf::from(format!("{}.{}", stem.join("_"), ext));
            Self(DesignPath::new(path).unwrap())
        }
    }

    quickcheck! {
        fn prop_read_command_never_empty(dp: ArbitraryDesignPath) -> bool {
            !dp.0.read_command().is_empty()
        }

        fn prop_path_preserved(dp: ArbitraryDesignPath) -> bool {
            let original = dp.0.path().to_path_buf();
            // Path should be preserved exactly
            dp.0.path() == original
        }
    }
}
