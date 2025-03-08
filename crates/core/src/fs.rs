use std::path::{Path, PathBuf};

mod dir;
mod file;

pub use dir::Directory;
pub use file::File;

#[derive(Debug)]
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

    pub fn file(&self) -> Option<&File> {
        if let Self::File(file) = self {
            Some(file)
        } else {
            None
        }
    }

    pub fn into_file(self) -> Option<File> {
        if let Self::File(file) = self {
            Some(file)
        } else {
            None
        }
    }

    pub fn directory(&self) -> Option<&Directory> {
        if let Self::Directory(dir) = self {
            Some(dir)
        } else {
            None
        }
    }

    pub fn into_directory(self) -> Option<Directory> {
        if let Self::Directory(dir) = self {
            Some(dir)
        } else {
            None
        }
    }
}
