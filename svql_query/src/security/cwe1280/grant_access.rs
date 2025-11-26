use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/security/access_control/grant_access/rtlil/grant_access.il",
    name = "grant_access"
)]
pub struct GrantAccess<S: State> {
    pub usr_id: Wire<S>,
    pub correct_id: Wire<S>,
    pub grant: Wire<S>,
}

#[cfg(test)]
mod tests {
    use crate::{
        Search, instance::Instance, security::cwe1280::grant_access::GrantAccess, traits::Query,
    };
    use std::sync::OnceLock;
    use svql_common::{Config, Dedupe, MatchLength, YosysModule};
    use svql_driver::Driver;

    #[derive(Debug, Clone)]
    struct GrantAccessTestCase {
        name: &'static str,
        fixture_path: &'static str,
        module_name: &'static str,
        _description: &'static str,
        expected_matches: usize,
    }

    static GRANT_ACCESS_CASES: &[GrantAccessTestCase] = &[
        GrantAccessTestCase {
            name: "fixed",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v",
            module_name: "cwe1280_fixed",
            _description: "Secure version with proper validation",
            expected_matches: 1,
        },
        GrantAccessTestCase {
            name: "vuln",
            fixture_path: "examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v",
            module_name: "cwe1280_vuln",
            _description: "Vulnerable version with weak ID check",
            expected_matches: 1,
        },
    ];

    fn init_test_logger() {
        static INIT: OnceLock<()> = OnceLock::new();
        let _ = INIT.get_or_init(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_test_writer()
                .try_init();
        });
    }

    fn run_grant_access_case(
        driver: &Driver,
        config: &Config,
        case: &GrantAccessTestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let haystack = YosysModule::new(case.fixture_path, case.module_name)?;

        let (haystack_key, haystack_design) = driver.get_or_load_design(
            &haystack.path().display().to_string(),
            haystack.module_name(),
            &config.haystack_options,
        )?;

        let context = GrantAccess::<Search>::context(driver, &config.needle_options)?;
        let context = context.with_design(haystack_key.clone(), haystack_design);

        let results = GrantAccess::<Search>::query(
            &GrantAccess::<Search>::instantiate(Instance::root("grant_access".to_string())),
            driver,
            &context,
            &haystack_key,
            config,
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

    #[rstest::rstest]
    fn test_all_grant_access_cases(
        #[values("fixed", "vuln")] case_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        init_test_logger();

        let config = Config::builder()
            .match_length(MatchLength::First)
            .dedupe(Dedupe::None)
            .pattern_vars_match_design_consts(true)
            .build();

        let driver = Driver::new_workspace()?;
        let case = GRANT_ACCESS_CASES
            .iter()
            .find(|c| c.name == case_name)
            .unwrap();

        run_grant_access_case(&driver, &config, case)
    }
}
