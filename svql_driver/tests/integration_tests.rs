
#[cfg(test)]
mod integration_tests {
    use std::path::PathBuf;
    use svql_driver::Driver;


    lazy_static::lazy_static! {

        static ref COMB_D_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/comb_d_double_sdffe.v");
        static ref AND_Q_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v");
        static ref SEP_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sep_double_sdffe.v");
        static ref SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }

    fn load_driver_from(path: &str) -> Driver {
        let path = std::path::PathBuf::from(path);
        let file_stem = PathBuf::from(path.file_stem().expect("Failed to get file stem"));
        let name = file_stem.file_name().expect("Failed to get file name").to_str().expect("Failed to convert file name to str").to_string();
        Driver::new(path, name, None).expect("Failed to create driver")
    }

    #[test]
    fn test_double_sdffe_comb_needle_sep_hay() {
        let comb_search_matches = svql_driver::subgraph::find_subgraphs(&COMB_D_DOUBLE_SDFFE.design_as_ref(), &SEP_DOUBLE_SDFFE.design_as_ref());
        assert_eq!(comb_search_matches.len(), 0, "Expected 0 matches for needle comb, against haystack sep, got {}", comb_search_matches.len());
    }

    #[test]
    fn test_double_sdffe_sep_needle_comb_hay() {
        let sep_search_matches = svql_driver::subgraph::find_subgraphs(&SEP_DOUBLE_SDFFE.design_as_ref(), &COMB_D_DOUBLE_SDFFE.design_as_ref());
        assert_eq!(sep_search_matches.len(), 2, "Expected 2 matches for needle sep, against haystack comb, got {}", sep_search_matches.len());
    }

}

#[cfg(test)]
mod integration_tests_2 {
    use std::path::{Path, PathBuf};

    use super::*;
    use rstest::rstest;
    use svql_driver::Driver;

    lazy_static::lazy_static! {

        static ref COMB_D_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/comb_d_double_sdffe.v");
        static ref AND_Q_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v");
        static ref SEP_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/sep_double_sdffe.v");
        static ref SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }

    fn load_driver_from(path: &str) -> (Driver, PathBuf) {
        let path = std::path::PathBuf::from(path);
        let file_stem = PathBuf::from(path.file_stem().expect("Failed to get file stem"));
        let name = file_stem.file_name().expect("Failed to get file name").to_str().expect("Failed to convert file name to str").to_string();
        let driver = Driver::new(path, name, None).expect("Failed to create driver");
        (driver, file_stem)
    }

   
    #[rstest]
    #[case(&COMB_D_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, &SEP_DOUBLE_SDFFE.0, 0)]
    fn test_subgraph_matches(
        #[case] needle_tuple: &'static (Driver, PathBuf),
        #[case] haystack_tuple: &'static (Driver, PathBuf),
        #[case] expected: usize,
    ) {
        let (needle, needle_path) = needle_tuple;
        let (haystack, haystack_path) = haystack_tuple;
        let matches = svql_driver::subgraph::find_subgraphs(needle.design_as_ref(), haystack.design_as_ref());
        assert_eq!(matches.len(), expected, "Expected {} matches for needle {}, against haystack {}, got {}", expected, needle_path.display(), haystack_path.display(), matches.len());
    }
}