use eyre::Result;
use std::path::{Path, PathBuf};

pub struct FileServer {
    root_path: PathBuf,
}

impl FileServer {
    pub fn new(root: &str) -> Result<Self> {
        let can_path = std::fs::canonicalize(Path::new(root))?;

        Ok(FileServer {
            root_path: can_path,
        })
    }
}

impl super::Responder for FileServer {
    fn handle_request(&self, _request: &super::HttpRequest) -> String {
        String::from("FileServer works!")
    }
}
