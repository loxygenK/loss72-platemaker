#![deny(clippy::unwrap_used)]

use std::path::Path;

use articles::ArticlePage;
use loss72_platemaker_construct::{ConstructFile, Construction};
use loss72_platemaker_core::fs::{Directory, FileSystemError};

mod articles;

pub use articles::generate_article_html;

#[derive(Debug, thiserror::Error)]
pub enum WebsiteGenerationError {
    #[error("Path was not valid: {0}")]
    PathError(#[from] FileSystemError),

    #[error("These placeholder is invalid: {}", .0.join(", "))]
    InvalidPlaceholder(Vec<String>),

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type OutputResult<T> = Result<T, WebsiteGenerationError>;

#[derive(Debug)]
pub struct WebPageHtmlTemplates {
    pub article: String,
}

pub fn load_templates(template_dir: &Directory) -> OutputResult<WebPageHtmlTemplates> {
    let [article] = template_dir.get_files(&[&"_article.html"])?;

    Ok(WebPageHtmlTemplates {
        article: article.read_to_string()?,
    })
}

pub fn get_webpage_construction<'a>(articles: &'a [ArticlePage]) -> Construction<'a> {
    Construction {
        dir: Path::new(""),
        content: vec![],
        sub_dir: vec![Construction {
            dir: Path::new("articles"),
            content: articles.iter().map(ConstructFile::from).collect(),
            sub_dir: vec![],
        }],
    }
}
