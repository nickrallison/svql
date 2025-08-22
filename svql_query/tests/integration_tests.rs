// Delegate entirely to the generated test harness (one #[test] per TestCase).
include!(concat!(env!("OUT_DIR"), "/svql_query_generated_tests.rs"));
