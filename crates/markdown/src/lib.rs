#![deny(clippy::unwrap_used)]

use loss72_platemaker_core::{
    fs::{Directory, File},
    log,
    model::Article,
};
use parse::{ParseError, make_article_from_markdown};

mod emoji;
mod frontmatter;
mod parse;

#[derive(Debug, thiserror::Error)]
pub enum MarkdownProcessError {
    #[error("Error during I/O: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Error during parsing Markdown:\n{0}")]
    ParseError(ParseError),
}

pub fn is_markdown_path(file: &File) -> bool {
    file.path().extension().is_some_and(|ext| ext == "md")
}

pub fn parse_markdown(root: &Directory, file: File) -> Result<Article, MarkdownProcessError> {
    log!(
        step: "Parsing ./{}",
        &file
            .path()
            .strip_prefix(root.path())
            .unwrap_or(file.path())
            .display()
            .to_string()
    );

    make_article_from_markdown(root.path(), file.path(), &file.read_to_string()?)
        .map_err(MarkdownProcessError::ParseError)
}
