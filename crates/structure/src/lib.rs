use std::{ops::Deref, path::{Path, PathBuf}};

use loss72_platemaker_core::fs::{Directory, FSNode, File};

pub struct ContentDirectory<'dir> {
    pub dir: &'dir Directory,
    pub markdown_files: Vec<ArticleFile>,
    pub article_group: Vec<ArticleGroup>,
}

impl<'dir> ContentDirectory<'dir> {
    pub fn new(dir: &'dir Directory) -> Result<Self, std::io::Error> {
        let mut article_group = ArticleGroup::scan(&dir)?;
        article_group.sort();
        article_group.dedup();

        let markdown_files = article_group
            .iter()
            .map(|group| {
                Directory::new(dir.path().join(group.group_dir_path()))
                    .and_then(|dir|
                        dir
                            .try_iter_content()?
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()
                    )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .filter_map(|node| node.into_file())
            .filter_map(|file| ArticleFile::from_file(&file, dir))
            .collect::<Vec<_>>();

        Ok(Self {
            dir,
            markdown_files,
            article_group,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArticleGroup {
    pub year: usize,
    pub month: usize,
}

impl ArticleGroup {
    pub fn scan(root: &Directory) -> std::io::Result<Vec<ArticleGroup>> {
        Ok(root
            .try_iter_tree()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|node| node.into_directory())
            .filter_map(|dir| Self::from_path(dir.path().strip_prefix(root.path()).unwrap()))
            .filter(|(_, suffix)| suffix.len() == 0)
            .map(|(group, _)| group)
            .collect::<Vec<_>>())
    }

    pub fn group_dir_path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.year.to_string())
            .join(self.month.to_string())
    }

    pub fn group_dir_flat_path(&self) -> PathBuf {
        PathBuf::new().join(format!("{:0>4}{:0>2}", self.year, self.month))
    }

    fn from_path(value: &Path) -> Option<(Self, Vec<String>)> {
        let mut components = value
            .iter()
            .take(2)
            .filter_map(|cmp| cmp.to_str().and_then(|cmp| cmp.parse::<usize>().ok()));

        let year = components.next()?;
        let month = components.next()?;

        let suffix_components = value
            .iter()
            .skip(2)
            .map(|cmp| cmp.to_str().map(|str| str.to_string()))
            .collect::<Option<Vec<_>>>()?;

        Some((Self { year, month }, suffix_components))
    }
}

#[derive(Debug)]
pub struct ArticleGroupNode {
    pub group: ArticleGroup,
    pub node: FSNode,
    pub relative_path: PathBuf,
    pub suffix_components: Vec<String>,
}
impl ArticleGroupNode {
    pub fn from_node(node: FSNode, root: &Directory) -> Option<Self> {
        let relative_path = node.path().strip_prefix(root.path()).ok()?.to_path_buf();
        let (article_group, suffix_components) = ArticleGroup::from_path(&relative_path)?;

        Some(Self {
            group: article_group,
            node,
            relative_path,
            suffix_components,
        })
    }
}

#[derive(Debug)]
pub struct ArticleFile(pub ArticleGroupNode);

impl ArticleFile {
    pub fn from_file(file: &File, root: &Directory) -> Option<Self> {
        let file = ArticleGroupNode::from_node(file.clone().into(), root)?;

        if file.suffix_components.len() == 1
            && file
                .relative_path
                .extension()
                .is_some_and(|ext| ext == "md")
        {
            Some(Self(file))
        } else {
            None
        }
    }

    pub fn file(&self) -> &File {
        self.node
            .file()
            .expect("Node to be saved as `from_file` initializes")
    }
}

impl Deref for ArticleFile {
    type Target = ArticleGroupNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct AssetFile(ArticleGroupNode);

impl AssetFile {
    pub fn from_file(file: &File, root: &Directory) -> Option<Self> {
        let file = ArticleGroupNode::from_node(file.clone().into(), root)?;

        if file.suffix_components.len() > 2
            && file
                .suffix_components
                .get(0)
                .is_some_and(|folder| folder == "assets")
        {
            Some(Self(file))
        } else {
            None
        }
    }

    pub fn file(&self) -> &File {
        self.node
            .file()
            .expect("Node to be saved as `from_file` initializes")
    }
}

impl Deref for AssetFile {
    type Target = ArticleGroupNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct AssetRootDir(ArticleGroupNode);

impl AssetRootDir {
    pub fn from_dir(dir: &Directory, root: &Directory) -> Option<Self> {
        let dir = ArticleGroupNode::from_node(dir.clone().into(), root)?;

        if dir.suffix_components.len() == 1
            && dir
                .suffix_components
                .get(0)
                .is_some_and(|folder| folder == "assets")
        {
            Some(Self(dir))
        } else {
            None
        }
    }

    pub fn directory(&self) -> &Directory {
        self.node
            .directory()
            .expect("Node to be saved as `from_dir` initializes")
    }
}

impl Deref for AssetRootDir {
    type Target = ArticleGroupNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
