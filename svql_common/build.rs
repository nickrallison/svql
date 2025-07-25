// build.rs

fn main() {
    let bridges: Vec<&str> = vec!["src/config.rs", "src/matches.rs", "src/pattern.rs"];

    cxx_build::bridges(bridges).compile("svql_common");

    println!("cargo:rerun-if-changed=src/");
}
