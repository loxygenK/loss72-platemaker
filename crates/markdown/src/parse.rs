mod control;
mod full_service;
mod sub_parser;

use super::frontmatter::parse_toml_to_metadata;
use full_service::MarkdownParser;
use loss72_platemaker_core::model::Article;
use loss72_platemaker_structure::ArticleFile;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(
        "The file is at the invalid location. Expected markdown files to be placed at `./$year/$month/$day[-$num]_$slug.md`."
    )]
    InvalidStructure,

    #[error("The file is at the path where cannot be represented in UTF-8.")]
    InvalidPath,

    #[error(
        "No TOML frontmatter was found. Write TOML frontmatter wrapped with `+++` at the top of the markdown content."
    )]
    NoFrontmatter,

    #[error("The frontmatter could not be parsed or not valid metadata:\n{0}")]
    InvalidToml(String),
}

pub fn make_article_from_markdown(file: &ArticleFile, content: &str) -> ParseResult<Article> {
    let content = parse_markdown(content)?;
    let metadata = parse_toml_to_metadata(&content.frontmatter)?;

    Ok(Article {
        id: file.id.clone(),
        metadata,
        content: content.html,
    })
}

#[derive(Clone, Debug)]
struct ParsedContent {
    frontmatter: String,
    html: String,
}

fn parse_markdown(content: &str) -> ParseResult<ParsedContent> {
    let parsed = MarkdownParser::parse(content, pulldown_cmark::Options::all());

    Ok(ParsedContent {
        html: parsed.html().to_string(),
        frontmatter: parsed
            .frontmatter()
            .ok_or(ParseError::NoFrontmatter)?
            .to_string(),
    })
}
