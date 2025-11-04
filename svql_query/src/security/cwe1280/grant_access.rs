use svql_macros::netlist;

netlist! {
    name: GrantAccess,
    module_name: "grant_access",
    file: "examples/patterns/security/access_control/grant_access/rtlil/grant_access.il",
    inputs: [usr_id, correct_id],
    outputs: [grant]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::sync::OnceLock;
    use svql_common::{Config, Dedupe, MatchLength, YosysModule};
    use svql_driver::Driver;

    use crate::{
        Search, instance::Instance, security::cwe1280::grant_access::GrantAccess,
        traits::netlist::SearchableNetlist,
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
    struct GrantAccessTestCase {
        name: &'static str,
        fixture_path: &'static str,
        module_name: &'static str,
        description: &'static str,
        expected_matches: usize,
    }

    static GRANT_ACCESS_CASES: &[GrantAccessTestCase] = &[
        GrantAccessTestCase {
            name: "fixed",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v",
            module_name: "cwe1280_fixed",
            description: "Fixed Grant Access",
            expected_matches: 1,
        },
        GrantAccessTestCase {
            name: "vuln",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v",
            module_name: "cwe1280_vuln",
            description: "Vulnerable Grant Access",
            expected_matches: 1,
        },
    ];

    #[rstest]
    #[case::fixed(GRANT_ACCESS_CASES[0].clone())]
    #[case::vuln(GRANT_ACCESS_CASES[1].clone())]
    fn test_all_grant_access_cases(#[case] case: GrantAccessTestCase) {
        init_test_logger();

        let config = Config::builder()
            .match_length(MatchLength::First)
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
                "✓ {}: {} ({} matches)",
                case.name, case.description, case.expected_matches
            );
        }
    }

    /// Helper: Run a single fixture-based test case, handling missing files gracefully
    fn run_single_case(
        driver: &Driver,
        config: &Config,
        case: &GrantAccessTestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture_path = case.fixture_path;

        let haystack_module = YosysModule::new(fixture_path, case.module_name)?;

        let (haystack_key, haystack_design) = driver.get_or_load_design(
            &haystack_module.path().display().to_string(),
            haystack_module.module_name(),
            &config.haystack_options,
        )?;

        let context = GrantAccess::<Search>::context(&driver, &config.needle_options)?;
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let results = GrantAccess::<Search>::query(
            &haystack_key,
            &context,
            Instance::root("grant".to_string()),
            &config,
        );

        assert_eq!(
            results.len(),
            case.expected_matches,
            "Expected {} matches for {}, got {}",
            case.expected_matches,
            case.name,
            results.len()
        );

        Ok(())
    }
}
