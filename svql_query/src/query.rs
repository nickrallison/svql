use std::path::{Path, PathBuf};
use crate::file_info::Match;

pub trait Module {
    fn file_path(&self) -> &PathBuf;
    fn query(&self, rtlil: &Path, top: String) -> Vec<Match> {
        // Run yosys and capture the output
        let file_path = self.file_path();
        
        let mut cmd = std::process::Command::new("yosys");
        let mut cmd = cmd
            .arg("-m")
            .arg("svql_driver.so")
            .arg("-p")
            .arg(format!("read {}", file_path.to_string_lossy()))
            .arg("-p")
            .arg(format!("hierarchy -top {}", top))
            .arg(format!("svql_driver -pat {}", file_path.display()));
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
    fn query(&self, rtlil: &Path, top: String) -> Vec<crate::file_info::Match>;
}

impl<T: Module> Query for T {
    fn query(&self, rtlil: &Path, top: String) -> Vec<Match> {
        todo!()
    }
}
