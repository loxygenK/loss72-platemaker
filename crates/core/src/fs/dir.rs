use std::{
    iter::Peekable,
    path::{Path, PathBuf},
};

use super::{FSNode, File, FileSystemError, FileSystemResult};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Directory(PathBuf);

impl Directory {
    pub fn new(path: impl AsRef<Path>) -> FileSystemResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FileSystemError::DoesNotExist(path.to_path_buf()));
        }

        let path = path.canonicalize()?;

        if !path.is_dir() {
            return Err(FileSystemError::EntryTypeMismatch(
                path.to_path_buf(),
                "file",
            ));
        }

        Ok(Directory(path.to_path_buf()))
    }

    pub fn new_with_mkdir(
        path: impl AsRef<Path>,
    ) -> Result<FileSystemResult<Self>, std::io::Error> {
        let path = path.as_ref().canonicalize()?;

        if !path.exists() {
            std::fs::create_dir(&path)?;
        }

        Ok(Directory::new(path))
    }

    pub fn new_unchecked(path: impl AsRef<Path>) -> Self {
        Directory(path.as_ref().to_path_buf())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn get_or_mkdir_child(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Result<Directory, FileSystemError>, std::io::Error> {
        Directory::new_with_mkdir(self.path().join(path))
    }

    pub fn get_file(&self, path: impl AsRef<Path>) -> FileSystemResult<File> {
        File::new(self.path().join(path))
    }

    pub fn get_files<const N: usize>(
        &self,
        paths: &[&impl AsRef<Path>; N],
    ) -> FileSystemResult<[File; N]> {
        let mut files: [Option<File>; N] = [const { None }; N];
        for (i, path) in paths.iter().enumerate() {
            files[i] = Some(self.get_file(path)?);
        }
        Ok(files.map(|file| file.unwrap()))
    }

    pub fn get_files_vec(&self, files: &[&impl AsRef<Path>]) -> FileSystemResult<Vec<File>> {
        files
            .iter()
            .map(|path| File::new(path.as_ref()))
            .collect::<Result<_, _>>()
    }

    pub fn iter_content(&self) -> Result<
        impl Iterator<Item = Result<FSNode, FileSystemError>>,
        std::io::Error
    > {
        Ok(self.path()
            .read_dir()?
            .flat_map(|entry| entry.map(|entry| {
                let path = entry.path();

                if path.is_file() {
                    File::new(&path).map(FSNode::File)
                } else if path.is_dir() {
                    Directory::new(&path).map(FSNode::Directory)
                } else {
                    Ok(FSNode::Unknown(path.to_path_buf()))
                }
            })))
    }

    pub fn try_iter_tree(&self) -> Result<RecursiveIterator, std::io::Error> {
        RecursiveIterator::new(self.path())
    }
}

pub struct RecursiveIterator {
    current: Peekable<std::fs::ReadDir>,
    child: Option<Box<RecursiveIterator>>,
}

impl RecursiveIterator {
    fn new(dir: &Path) -> Result<Self, std::io::Error> {
        Ok(Self {
            current: dir.read_dir()?.peekable(),
            child: None,
        })
    }
}

impl Iterator for RecursiveIterator {
    type Item = Result<File, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child_next) = self.child.as_mut().and_then(|child| child.next()) {
            return Some(child_next);
        }

        for file in &mut self.current {
            let file = match file {
                Ok(file) => file,
                Err(err) => return Some(Err(err)),
            };

            let path = file.path();
            if path.is_dir() {
                self.child = match RecursiveIterator::new(&path) {
                    Ok(iter) => Some(Box::new(iter)),
                    Err(err) => return Some(Err(err)),
                };
                return self.child.as_mut().unwrap().next();
            }

            if path.is_file() {
                return Some(Ok(File::new(&path).unwrap()));
            }
        }

        None
    }
}
