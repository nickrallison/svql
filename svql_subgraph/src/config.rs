#[derive(Clone, Debug)]
pub struct Config {
    pub match_length: bool,
}

impl Config {
    pub fn new(match_length: bool) -> Self {
        Self { match_length }
    }
}
