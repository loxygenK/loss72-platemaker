#![deny(clippy::unwrap_used)]

use loss72_platemaker_core::{fs::File, log, model::Article};
use loss72_platemaker_structure::ArticleFile;
use parse::{ParseError, make_article_from_markdown};

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

pub fn parse_markdown(file: &ArticleFile) -> Result<Article, MarkdownProcessError> {
    log!(step: "Parsing ./{}", file.relative_path.display());

    make_article_from_markdown(file, &file.file().read_to_string()?)
        .map_err(MarkdownProcessError::ParseError)
}
