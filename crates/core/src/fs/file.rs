use std::path::{Path, PathBuf};

use super::{Directory, FileSystemError, FileSystemResult};

#[derive(Debug)]
pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: impl AsRef<Path>) -> FileSystemResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FileSystemError::DoesNotExist(path.to_path_buf()));
        }

        let path = path.canonicalize()?;

        if !path.is_file() {
            return Err(FileSystemError::EntryTypeMismatch(
                path.to_path_buf(),
                "file",
            ));
        }

        Ok(File {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn containing_dir(&self) -> Option<FileSystemResult<Directory>> {
        self.path.parent().map(Directory::new)
    }

    pub fn read_to_string(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }
}
