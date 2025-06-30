use std::path::PathBuf;

pub trait Graph {
    fn get_pattern_path() -> PathBuf;
}

pub trait GraphImpl {
    fn get_matches();
}

impl<T: Graph> GraphImpl for T {
    fn get_matches() {
        // Call Yosys SVQL Plugin
    }
}