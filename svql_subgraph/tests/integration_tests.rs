#[cfg(test)]
mod svql_subgraph_generated_tests {
    include!(concat!(
        env!("OUT_DIR"),
        "/svql_subgraph_generated_tests.rs"
    ));
}
