use loss72_platemaker_core::model::ArticleMetadata;

use super::parse::{ParseError, ParseResult};

pub fn parse_toml_to_metadata(toml: &str) -> ParseResult<ArticleMetadata> {
    toml::from_str::<ArticleMetadata>(toml).map_err(|e| ParseError::InvalidToml(format!("{e}")))
}
