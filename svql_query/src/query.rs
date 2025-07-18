use std::path::{Path, PathBuf};
use svql_common::mat::Match;

pub trait Module {
    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> String;
    fn query_module<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match> {
        // Run yosys and capture the output
        let needle_file_path: PathBuf = self.file_path();
        let haystack_file_path: PathBuf = design_path.into();
        let needle_module_name: String = self.module_name();
        let haystack_module_name: String = top.into();
        
        let mut cmd = std::process::Command::new("./yosys/yosys");
        let cmd = cmd
            .arg("-m")
            .arg("build/svql_driver/libsvql_driver.so")
            .arg("-p")
            .arg(format!("read_verilog {}", haystack_file_path.to_string_lossy()))
            .arg("-p")
            .arg(format!("hierarchy -top {}", haystack_module_name))
            .arg("-p")
            .arg("proc")
            .arg("-p")
            .arg(format!("svql_driver -pat {} {} -verbose", needle_file_path.display(), needle_module_name));
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
    fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match>;
}

impl<T: Module> Query for T {
    fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match> {
        self.query_module(design_path.into(), top.into())
    }
}
