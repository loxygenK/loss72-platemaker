use std::{
    iter::Peekable,
    path::{Path, PathBuf},
};

use super::{FSNode, File};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Directory(PathBuf);

impl Directory {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();

        let path = path.canonicalize()?;

        if !path.is_dir() {
            return Err(std::io::ErrorKind::NotADirectory.into());
        }

        Ok(Directory(path.to_path_buf()))
    }

    pub fn new_with_mkdir(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }

        Directory::new(path)
    }

    pub fn new_unchecked(path: impl AsRef<Path>) -> Self {
        Directory(path.as_ref().to_path_buf())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn get_child(&self, path: impl AsRef<Path>) -> Option<std::io::Result<Directory>> {
        let child_path = self.path().join(path);
        if child_path.exists() {
            Some(Directory::new(child_path))
        } else {
            None
        }
    }

    pub fn get_or_mkdir_child(&self, path: impl AsRef<Path>) -> std::io::Result<Directory> {
        Directory::new_with_mkdir(self.path().join(path))
    }

    pub fn get_file(&self, path: impl AsRef<Path>) -> std::io::Result<File> {
        File::new(self.path().join(path))
    }

    pub fn get_files<const N: usize>(
        &self,
        paths: &[&impl AsRef<Path>; N],
    ) -> std::io::Result<[File; N]> {
        Ok(paths
            .iter()
            .map(|path| self.get_file(path))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .expect("paths to be initialized in N-sized array as type parameter notes"))
    }

    pub fn get_files_vec(&self, files: &[&impl AsRef<Path>]) -> std::io::Result<Vec<File>> {
        files
            .iter()
            .map(|path| File::new(path.as_ref()))
            .collect::<Result<_, _>>()
    }

    pub fn try_iter_content(
        &self,
    ) -> Result<impl Iterator<Item = std::io::Result<FSNode>>, std::io::Error> {
        Ok(self.path().read_dir()?.flat_map(|entry| {
            entry.map(|entry| {
                let path = entry.path();

                if path.is_file() {
                    File::new(&path).map(FSNode::File)
                } else if path.is_dir() {
                    Directory::new(&path).map(FSNode::Directory)
                } else {
                    Ok(FSNode::Unknown(path.to_path_buf()))
                }
            })
        }))
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
    type Item = Result<FSNode, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child_next) = self.child.as_mut().and_then(|child| child.next()) {
            return Some(child_next);
        }

        let node = match self.current.next()? {
            Ok(node) => node,
            Err(err) => return Some(Err(err)),
        };

        let path = node.path();

        if path.is_file() {
            let file = File::new(&path).expect("entry to be exist and file");
            Some(Ok(FSNode::File(file)))
        } else if path.is_dir() {
            self.child = Some(Box::new(match RecursiveIterator::new(&path) {
                Ok(iter) => iter,
                Err(err) => return Some(Err(err)),
            }));

            let dir = Directory::new(&path).expect("entry to be exist and file");
            Some(Ok(FSNode::Directory(dir)))
        } else {
            Some(Ok(FSNode::Unknown(path.to_path_buf())))
        }
    }
}

impl From<Directory> for FSNode {
    fn from(dir: Directory) -> Self {
        FSNode::Directory(dir)
    }
}
