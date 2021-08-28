use std::path::PathBuf;

pub enum Installation {
    Valid { path: PathBuf, version: String },
    Incomplete { path: PathBuf, version: String },
}
