use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunboxError {
    #[error("VFS error: {0}")]
    Vfs(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("shell error: {0}")]
    Shell(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, RunboxError>;
