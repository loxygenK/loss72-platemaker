use std::path::{Path, PathBuf};

pub const ARTICLE_TEMPLATE: &str = "_article.html";
pub const INDEX_TEMPLATE: &str = "_index.html";
pub const INDEX_LIST_TEMPLATE: &str = "_index-list.html";

pub const TEMPLATE_FILES: [&str; 3] = [ARTICLE_TEMPLATE, INDEX_TEMPLATE, INDEX_LIST_TEMPLATE];

pub fn template_file_paths() -> [PathBuf; 3] {
    TEMPLATE_FILES.map(PathBuf::from)
}

pub fn is_template_file(path: &Path) -> bool {
    TEMPLATE_FILES.iter().any(|template| path.file_name().is_some_and(|file| &file == template))
}

