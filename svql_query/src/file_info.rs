use std::path::PathBuf;

pub struct FileInfo {
    file_name: PathBuf,
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,

}
pub struct Match {
    file_infos: Vec<FileInfo>,
}