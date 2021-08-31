use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    Critical(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Critical(msg) => write!(f, "critical error: {}", msg),
        }
    }
}
