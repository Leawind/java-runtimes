use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Error {
    pub(crate) kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    InvalidWorkDir,
    NoJavaVersionStringFound,
    LooksNotLikeJavaExecutableFile(PathBuf),
    JavaOutputFailed(std::io::Error),
    GettingJavaVersionFailed(PathBuf),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::InvalidWorkDir => write!(f, "Java home directory not found"),
            ErrorKind::NoJavaVersionStringFound => write!(f, "Invalid version string"),
            ErrorKind::LooksNotLikeJavaExecutableFile(path) => {
                write!(
                    f,
                    "Path looks not like a Java executable file [**/bin/java(.exe)] : {}",
                    path.display()
                )
            }
            ErrorKind::JavaOutputFailed(io_err) => {
                write!(f, "Failed to read Java output: {}", io_err)
            }
            ErrorKind::GettingJavaVersionFailed(path) => {
                write!(f, "Failed to get Java version: {}", path.display())
            }
        }
    }
}
