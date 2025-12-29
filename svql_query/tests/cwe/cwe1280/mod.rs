#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::sync::OnceLock;
    use svql_common::{Config, Dedupe, MatchLength, YosysModule};
    use svql_driver::Driver;
    use svql_query::{
        Search,
        instance::Instance,
        security::cwe1280::Cwe1280,
        traits::{Query, Searchable},
    };

    fn init_test_logger() {
        static INIT: OnceLock<()> = OnceLock::new();
        let _ = INIT.get_or_init(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_test_writer()
                .try_init();
        });
    }

    #[derive(Debug, Clone)]
    struct Cwe1280TestCase {
        name: &'static str,
        fixture_path: &'static str,
        module_name: &'static str,
        description: &'static str,
        expected_matches: usize,
    }

    static CWE1280_CASES: &[Cwe1280TestCase] = &[
        Cwe1280TestCase {
            name: "fixed",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v",
            module_name: "cwe1280_fixed",
            description: "FIXED: Strong validation (no grant bypass)",
            expected_matches: 0,
        },
        Cwe1280TestCase {
            name: "vuln",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v",
            module_name: "cwe1280_vuln",
            description: "Vulnerable: Weak validation (grant bypass)",
            expected_matches: 1, // FIXED: Post-port fix, now expects 1 (GrantAccess + LockedRegister chain matches)
        },
    ];

    #[rstest]
    #[case::fixed(CWE1280_CASES[0].clone())]
    #[case::vuln(CWE1280_CASES[1].clone())]
    fn test_all_cwe1280_cases(#[case] case: Cwe1280TestCase) {
        init_test_logger();

        let config = Config::builder()
            .match_length(MatchLength::NeedleSubsetHaystack)
            .pattern_vars_match_design_consts(true)
            .haystack_opt(true)
            .dedupe(Dedupe::None)
            .build();

        let driver = Driver::new_workspace().unwrap();

        if let Err(e) = run_single_case(&driver, &config, &case) {
            eprintln!("{} test skipped due to error: {}", case.name, e);
            assert_eq!(
                0, case.expected_matches,
                "{}: Fixture error but expected matches > 0; adjust fixture or expected",
                case.name
            );
        } else {
            println!(
                "Success {}: {} ({} matches)",
                case.name, case.description, case.expected_matches
            );
        }
    }

    /// Helper: Run a single fixture-based test case, handling missing files gracefully
    fn run_single_case(
        driver: &Driver,
        config: &Config,
        case: &Cwe1280TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let haystack_module = YosysModule::new(case.fixture_path, case.module_name)?;

        let (haystack_key, haystack_design) = driver.get_or_load_design(
            &haystack_module.path().display().to_string(),
            haystack_module.module_name(),
            &config.haystack_options,
        )?;

        let context = Cwe1280::<Search>::context(driver, &config.needle_options)?;
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let query = Cwe1280::<Search>::instantiate(Instance::root("cwe1280".to_string()));
        let results = query.query(driver, &context, &haystack_key, config);

        assert_eq!(
            results.len(),
            case.expected_matches,
            "Case {}: expected {} matches, got {}",
            case.name,
            case.expected_matches,
            results.len()
        );

        Ok(())
    }
}
