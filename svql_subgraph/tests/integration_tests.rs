#[cfg(test)]
mod tests {
    use log::{LevelFilter, trace};
    use std::sync::Once;
    use svql_common::{ALL_TEST_CASES, Pattern, import_design};

    // Initialize env_logger only once and integrate with the test harness capture.
    // The per-case log enabling/disabling is handled via log::set_max_level.
    static INIT_LOGGER: Once = Once::new();

    fn init_test_logger() {
        INIT_LOGGER.call_once(|| {
            // Use test mode so output is captured by the test harness.
            let mut b = env_logger::Builder::from_default_env();
            b.is_test(true);

            // You can keep verbose per-module filters here if you like,
            // they will still be bounded by log::set_max_level.
            // Example:
            // b.filter_module("svql_subgraph", LevelFilter::Trace)
            //     .filter_module("prjunnamed_netlist", LevelFilter::Trace);

            // Default baseline doesn't matter much because we will clamp with set_max_level below.
            b.filter_level(LevelFilter::Trace);

            let _ = b.try_init(); // ignore if already initialized
        });
    }

    // Run a closure with a temporary max log level, restoring the previous level after.
    fn with_max_level<T>(level: LevelFilter, f: impl FnOnce() -> T) -> T {
        let prev = log::max_level();
        log::set_max_level(level);
        let out = f();
        log::set_max_level(prev);
        out
    }

    // Run a single TestCase. This is your original logic extracted.
    fn run_one(test_case: &svql_common::TestCase) -> Result<(), Box<dyn std::error::Error>> {
        let (netlist_pattern_path, netlist_pattern_module) = match test_case.pattern {
            Pattern::Netlist { path, module, .. } => (path, module),
            _ => panic!("Test case is not a netlist pattern"),
        };

        let needle = import_design(netlist_pattern_path.into(), netlist_pattern_module)
            .expect(&format!("Failed to load needle: {}", netlist_pattern_path));
        let haystack =
            import_design(test_case.haystack.path.into(), test_case.haystack.module).expect(
                &format!("Failed to load haystack: {}", test_case.haystack.path),
            );

        let matches = svql_subgraph::find_subgraphs(&needle, &haystack, &test_case.config);

        let expected_matches = test_case.expected_matches;
        let actual_matches = matches.len();

        if actual_matches != expected_matches {
            trace!(
                "Test case '{}' failed: expected {} matches, got {}",
                test_case.name, expected_matches, actual_matches
            );
            return Err(format!(
                "Test case '{}' failed: expected {} matches, got {}",
                test_case.name, expected_matches, actual_matches
            )
            .into());
        }
        Ok(())
    }

    fn run_quiet_then_loud_on_fail(
        test_case: &svql_common::TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        init_test_logger();

        // Force logs if test_case.name contains the provided substring
        let force_log = std::env::var("SVQL_LOG_CASE")
            .ok()
            .map(|needle| test_case.name.contains(&needle))
            .unwrap_or(false);

        if force_log {
            eprintln!(
                "SVQL_LOG_CASE matched. Running '{}' with logs enabled.",
                test_case.name
            );
            return with_max_level(log::LevelFilter::Trace, || run_one(test_case));
        }

        // First pass: silence logs
        let result = with_max_level(log::LevelFilter::Off, || run_one(test_case));

        // On failure, re-run with logs so only this case emits
        if result.is_err() {
            eprintln!("Re-running '{}' with logs enabled...", test_case.name);
            let _ = with_max_level(log::LevelFilter::Trace, || run_one(test_case));
            // panic!("DEBUGGING");
            return result;
        }

        result
    }

    #[test]
    fn test_all_netlist_cases() {
        // Run all cases, but only print logs for failing ones (and any forced via SVQL_LOG_CASE).
        let test_cases = ALL_TEST_CASES.iter().filter(|tc| tc.pattern.is_netlist());

        // Keep behavior similar to original: run all, collect errors, report at the end.
        let results: Vec<(&str, Result<(), Box<dyn std::error::Error>>)> = test_cases
            .map(|tc| (tc.name, run_quiet_then_loud_on_fail(tc)))
            .collect();

        let num_errors = results.iter().filter(|(_, r)| r.is_err()).count();

        if num_errors > 0 {
            eprintln!("{} out of {} tests failed", num_errors, results.len());
            for (name, result) in results {
                if let Err(e) = result {
                    eprintln!("Test '{}' failed: {}", name, e);
                }
            }
            // Fail the overall test
            assert!(false, "Some test cases failed");
        }
    }
}
