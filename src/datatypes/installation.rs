use std::path::PathBuf;

#[derive(Debug)]
pub enum Installation {
    Valid { path: PathBuf, version: String },
    Incomplete { path: PathBuf, version: String },
}
