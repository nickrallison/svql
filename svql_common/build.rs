use cbindgen::{Builder, Language};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    // rebuild triggers ------------------------------------------------------
    // println!("cargo:rerun-if-changed=src/");
    // println!("cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR");

    // let crate_dir =
    //     PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    // let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    // let include_dir = out_dir.join("include");
    // let header_path = include_dir.join("source.h");

    // Builder::new()
    //     .with_crate(&crate_dir)
    //     .with_language(Language::C)
    //     .with_include_guard("SVQL_COMMON_SOURCE_H")
    //     .generate()
    //     .expect("cbindgen failed")
    //     .write_to_file(&header_path);
}
