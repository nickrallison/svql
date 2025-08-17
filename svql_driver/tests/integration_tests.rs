
#[cfg(test)]
mod integration_tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use rstest::rstest;
    use svql_driver::Driver;

    lazy_static::lazy_static! {

        static ref COMB_D_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/comb_d_double_sdffe.v");
        static ref AND_Q_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v");
        static ref PAR_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/par_double_sdffe.v");
        static ref SEQ_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/seq_double_sdffe.v");
        static ref SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }

    fn load_driver_from(path: &str) -> (Driver, PathBuf) {
        let path = std::path::PathBuf::from(path);
        let name = PathBuf::from(path.file_stem().expect("Failed to get file stem"));
        let driver = Driver::new(path, name.display().to_string(), None).expect("Failed to create driver");
        (driver, name)
    }

   
    #[rstest]
    // Comb D Needle
    #[case(&COMB_D_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
    #[case(&COMB_D_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 0)]
    #[case(&COMB_D_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 0)]
    #[case(&COMB_D_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 0)]
    #[case(&COMB_D_DOUBLE_SDFFE, &SDFFE, 0)]
    // And Q Needle
    #[case(&AND_Q_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 0)]
    #[case(&AND_Q_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
    #[case(&AND_Q_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 0)]
    #[case(&AND_Q_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 0)]
    #[case(&AND_Q_DOUBLE_SDFFE, &SDFFE, 0)]
    // Par Double Needle
    #[case(&PAR_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
    #[case(&PAR_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
    #[case(&PAR_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 2)]
    #[case(&PAR_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 0)]
    #[case(&PAR_DOUBLE_SDFFE, &SDFFE, 0)]
    // Seq Double Needle
    #[case(&SEQ_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 0)]
    #[case(&SEQ_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 0)]
    #[case(&SEQ_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 2)]
    #[case(&SEQ_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 1)]
    #[case(&SEQ_DOUBLE_SDFFE, &SDFFE, 0)]
    // SDFFE Needle
    #[case(&SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
    #[case(&SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
    #[case(&SDFFE, &PAR_DOUBLE_SDFFE, 2)]
    #[case(&SDFFE, &SEQ_DOUBLE_SDFFE, 2)]
    #[case(&SDFFE, &SDFFE, 1)]
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