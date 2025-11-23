// //! Build support utilities for SVQL.
// //!
// //! This module provides helpers for build scripts, such as extracting test case names
// //! and sanitizing identifiers.

// use regex::Regex;

// /// Extract all TestCase.name literals from svql_common/src/test_cases.rs.
// /// Uses include_str! so build scripts can discover names without file IO jitter.
// pub fn test_case_names() -> Vec<String> {
//     // This file sits next to us in src/
//     const TEST_CASES_RS: &str = include_str!("test_cases.rs");
//     let re = Regex::new(r#"TestCase\s*\{\s*name:\s*"([^"]+)""#).unwrap();
//     re.captures_iter(TEST_CASES_RS)
//         .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
//         .collect()
// }

// /// Sanitize a string into something valid as a Rust identifier.
// /// Matches the logic previously used in build scripts.
// pub fn sanitize_ident(s: &str) -> String {
//     let mut out = String::with_capacity(s.len());
//     for (i, ch) in s.chars().enumerate() {
//         let valid = ch.is_ascii_alphanumeric() || ch == '_';
//         if (i == 0 && ch.is_ascii_digit()) || !valid {
//             out.push('_');
//         } else {
//             out.push(ch);
//         }
//     }
//     out
// }
