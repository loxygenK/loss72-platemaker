use std::{io::ErrorKind, path::{Path, PathBuf}};

use super::{Directory, FSNode};

#[derive(Clone, Debug)]
pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ErrorKind::NotFound.into());
        }

        let path = path.canonicalize()?;

        if !path.is_file() {
            return Err(ErrorKind::IsADirectory.into());
        }

        Ok(File {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn containing_dir(&self) -> Option<std::io::Result<Directory>> {
        self.path.parent().map(Directory::new)
    }

    pub fn read_to_string(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }
}

impl From<File> for FSNode {
    fn from(file: File) -> Self {
        FSNode::File(file)
    }
}
