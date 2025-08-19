#[derive(Clone, Debug)]
pub struct Config {
    pub match_length: bool,
    pub dedupe: DedupeMode,
}

impl Config {
    pub fn new(match_length: bool, dedupe: DedupeMode) -> Self {
        Self {
            match_length,
            dedupe,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DedupeMode {
    /// Original behavior: include boundary bindings in the dedupe signature.
    Full,
    /// Collapse matches that share the same mapped gate set, ignoring boundary bindings.
    GatesOnly,
}
