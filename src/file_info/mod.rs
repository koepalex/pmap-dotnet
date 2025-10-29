use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    path: PathBuf,
}

impl FileInfo {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn read_to_string(&self) -> std::io::Result<String> {
        fs::read_to_string(&self.path)
    }

    pub fn full_name(&self) -> String {
        self.path.to_str().unwrap_or("").to_string()
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }
}