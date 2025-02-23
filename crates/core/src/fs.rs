use std::path::{Path, PathBuf};

mod dir;
mod file;

pub use dir::Directory;
pub use file::File;

pub enum FSNode {
    File(File),
    Directory(Directory),
    Unknown(PathBuf),
}

impl FSNode {
    pub fn path(&self) -> &Path {
        match self {
            FSNode::File(file) => file.path(),
            FSNode::Directory(directory) => directory.path(),
            FSNode::Unknown(path_buf) => path_buf.as_path(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("{0}: Does not exist")]
    DoesNotExist(PathBuf),

    #[error("{0}: Expected the entry to be the type {1}, but it was not")]
    EntryTypeMismatch(PathBuf, &'static str),

    #[error("The path could not be canonicalized: {0}")]
    PathCanonicalizeFailure(#[from] std::io::Error),
}

pub type FileSystemResult<T> = Result<T, FileSystemError>;
