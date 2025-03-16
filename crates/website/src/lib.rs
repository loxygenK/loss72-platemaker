#![deny(clippy::unwrap_used)]

use std::path::Path;

use articles::{IndexPage, ArticlePage};
use loss72_platemaker_construct::{ConstructFile, Construction};
use loss72_platemaker_core::fs::Directory;

mod articles;

pub use articles::{generate_article_html, generate_index_html};

#[derive(Debug, thiserror::Error)]
pub enum WebsiteGenerationError {
    #[error("These placeholder is invalid: {}", .0.join(", "))]
    InvalidPlaceholder(Vec<String>),

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type OutputResult<T> = Result<T, WebsiteGenerationError>;

#[derive(Debug)]
pub struct WebPageHtmlTemplates {
    pub article: String,
    pub index: String,
    pub index_style: String,
    pub index_list: String,
}

pub fn load_templates(template_dir: &Directory) -> OutputResult<WebPageHtmlTemplates> {
    let [article, index, index_list] = template_dir.get_files(&[&"_article.html", &"_index.html", &"_index-list.html"])?;
    let [index_style] = template_dir.get_child(Path::new("styles"))
        .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))??
        .get_files(&[&"index.css"])?;

    Ok(WebPageHtmlTemplates {
        article: article.read_to_string()?,
        index: index.read_to_string()?,
        index_style: index_style.read_to_string()?,
        index_list: index_list.read_to_string()?,
    })
}

pub fn get_webpage_construction<'a>(index: Option<&'a IndexPage>, articles: &'a [ArticlePage]) -> Construction<'a> {
    Construction {
        dir: Path::new(""),
        content: if let Some(index) = index { vec![index.into()] } else { vec![] },
        sub_dir: vec![Construction {
            dir: Path::new("articles"),
            content: articles.iter().map(ConstructFile::from).collect(),
            sub_dir: vec![],
        }],
    }
}
