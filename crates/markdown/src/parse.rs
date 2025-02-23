use std::path::{Component, Path};

use super::{emoji::replace_emoji_from_shortcode, frontmatter::parse_toml_to_metadata};
use loss72_platemaker_core::model::Article;
use markdown::{Options, mdast};

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

pub fn make_article_from_markdown(root: &Path, path: &Path, content: &str) -> ParseResult<Article> {
    let (group, slug) = path_into_group_and_slug(root, path)?;
    let content = parse_markdown(content)?;
    let metadata = parse_toml_to_metadata(&content.frontmatter)?;

    let content = replace_emoji_from_shortcode(&content.html);

    Ok(Article {
        group,
        slug,
        metadata,
        content,
    })
}

pub fn path_into_group_and_slug(root: &Path, path: &Path) -> ParseResult<(String, String)> {
    assert!(path.starts_with(root),);

    let relative_path = path.strip_prefix(root).unwrap_or_else(|_| {
        panic!(
            "Path is not sub of root\n  Root = {}\n  Path = {}",
            root.to_string_lossy(),
            path.to_string_lossy()
        )
    });

    let components = relative_path
        .components()
        .skip_while(|component| !matches!(component, Component::Normal(_)))
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or(ParseError::InvalidPath)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let &[year, month, day_and_slug] = components.as_slice() else {
        println!("{:#?}", components.as_slice());
        return Err(ParseError::InvalidStructure);
    };

    let day_and_slug = day_and_slug.strip_suffix(".md").unwrap_or(day_and_slug);

    Ok((
        format!("{:0>4}{:0>2}", year, month),
        day_and_slug.to_string(),
    ))
}

#[derive(Clone, Debug)]
struct ParsedContent {
    frontmatter: String,
    html: String,
}

fn parse_markdown(content: &str) -> ParseResult<ParsedContent> {
    let options: Options = {
        let mut gfm_default = Options::gfm();
        gfm_default.parse.constructs.frontmatter = true;
        gfm_default.compile.allow_dangerous_html = true;
        gfm_default.compile.allow_dangerous_protocol = true;

        gfm_default
    };

    // Safety: Markdown is always parsable (but MDX is not - we are only parsing Markdown),
    //         so the Result is practically infallible here
    let html =
        markdown::to_html_with_options(content, &options).expect("Markdown to be always parsable");
    let node = markdown::to_mdast(content, &options.parse).expect("Markdown to be always parsable");

    let mdast::Node::Root(root) = node else {
        panic!("Expected the topmost node is Root (developer oversight)");
    };
    let frontmatter = root
        .children
        .into_iter()
        .find_map(|node| {
            if let mdast::Node::Toml(toml) = node {
                Some(toml)
            } else {
                None
            }
        })
        .ok_or(ParseError::NoFrontmatter)?
        .value;

    Ok(ParsedContent { html, frontmatter })
}
