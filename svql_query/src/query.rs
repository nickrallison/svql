use std::path::{Path, PathBuf};
use crate::file_info::Match;

pub trait Module {
    fn file_path(&self) -> PathBuf;
    fn query_module<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match> {
        // Run yosys and capture the output
        let file_path: PathBuf = self.file_path();
        let design_path: PathBuf = design_path.into();
        let top: String = top.into();
        
        let mut cmd = std::process::Command::new("./yosys/yosys");
        let mut cmd = cmd
            .arg("-m")
            .arg("build/svql_driver/libsvql_driver.so")
            .arg("-p")
            .arg(format!("read_verilog {}", design_path.to_string_lossy()))
            .arg("-p")
            .arg(format!("hierarchy -top {}", top))
            .arg("-p")
            .arg("proc")
            .arg("-p")
            .arg(format!("svql_driver -pat {} -verbose", file_path.display()));
        let output = cmd
            .output()
            .expect("Failed to execute yosys command");

        if !output.status.success() {
            panic!("Yosys command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        let stdout = String::from_utf8(output.stdout)
            .expect("Failed to convert yosys output to string");

        println!("{}", stdout);
        todo!() // Here we would parse the output and return a Vec<Match>
    }
}



pub trait Query {
    fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<crate::file_info::Match>;
}

impl<T: Module> Query for T {
    fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match> {
        self.query_module(design_path.into(), top.into())
    }
}
