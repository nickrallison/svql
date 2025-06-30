use cmake::Config;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=.gitmodules");
    println!("cargo:rerun-if-changed=libs/");

    let status = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .arg("--recursive")
        .status()
        .expect("Failed to spawn `git submodule update`");
    if !status.success() {
        panic!(
            "`git submodule update --init --recursive` failed with exit code {:?}",
            status.code()
        );
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());

    todo!("Locate static library for svql_common and include directory");

    // let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // let artifacts_root = manifest_dir.join("../artifacts");
    // let svql_common_include = artifacts_root.join("include");
    // let lib_filename = "libsvql_common.a";

    // let svql_common_lib = artifacts_root.join("lib").join(lib_filename);

    // // Tell Cargo to rebuild if the header changes
    // println!("cargo:rerun-if-changed={}", svql_common_include.display());

    // // Invoke CMake, passing in both the library and the newly-found include dir
    // let _dst = Config::new("CMakeLists.txt")
    //     .define("CMAKE_BUILD_TYPE", &profile)
    //     .define("SVQL_COMMON_LIB", svql_common_lib.to_str().unwrap())
    //     .define("SVQL_COMMON_INCLUDE", svql_common_include.to_str().unwrap())
    //     .build();
}
