use loss72_platemaker_construct::ConstructFile;
use loss72_platemaker_core::{log, model::Article, util::get_slice_by_char};
use loss72_platemaker_template::Placeholder;
use std::{
    any::type_name,
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{OutputResult, WebPageHtmlTemplates, WebsiteGenerationError};

pub struct IndexPage {
    pub html: String,
    pub path: PathBuf,
}


impl<'p> From<&'p IndexPage> for ConstructFile<'p> {
    fn from(value: &'p IndexPage) -> Self {
        ConstructFile {
            path: &value.path,
            content: &value.html,
        }
    }
}

pub struct ArticlePage<'article> {
    pub article: &'article Article,
    pub html: String,
    pub path: PathBuf,
}

impl<'p> From<&'p ArticlePage<'_>> for ConstructFile<'p> {
    fn from(value: &'p ArticlePage<'_>) -> Self {
        ConstructFile {
            path: &value.path,
            content: &value.html,
        }
    }
}

impl std::fmt::Debug for ArticlePage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = type_name::<Self>();
        let type_name = type_name.rsplit("::").next().unwrap_or(type_name);

        f.debug_struct(type_name)
            .field("article", self.article)
            .field(
                "html",
                &format_args!(
                    "\"{} ... (truncated) \".",
                    get_slice_by_char(&self.html, 0..30).escape_debug()
                ),
            )
            .field("path", &self.path)
            .finish()
    }
}

pub fn generate_index_html(
    html_templates: &WebPageHtmlTemplates,
    article: &[ArticlePage],
) -> OutputResult<IndexPage> {
    log!(step: "Generating HTML for index page");

    let placeholder = Placeholder::from_strs("${", "}", None)
        .expect("Regex is validated to include the capture group");

    // We create list elements first
    let article_tag_iter = article.iter()
        .map(|page| {
            let placeholder_contents = HashMap::from([
                ("url", page.path.to_string_lossy().to_string()),
                ("title", page.article.metadata.title.clone()),
            ]);
            placeholder
                .partially_fill_placeholders(&html_templates.index_list, |name| {
                    placeholder_contents.get(name).cloned()
                })
                .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))
        })
        .collect::<Result<String, _>>()?;

    let placeholder_contents = HashMap::from([("articles", article_tag_iter)]);

    Ok(IndexPage {
        path: PathBuf::from("index.html"),
        html: placeholder
            .partially_fill_placeholders(&html_templates.index, |name| {
                placeholder_contents.get(name).cloned()
            })
            .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))?
    })
}

pub fn generate_article_html<'article>(
    html_templates: &WebPageHtmlTemplates,
    article: &'article Article,
) -> OutputResult<ArticlePage<'article>> {
    log!(step: "Generating HTML for slug '{}'", &article.slug);

    let placeholder = Placeholder::from_strs("${", "}", None)
        .expect("Regex is validated to include the capture group");

    let mut placeholder_contents = HashMap::from([
        ("title", article.metadata.title.clone()),
        ("slug", article.slug.clone()),
        ("content", article.content.clone()),
    ]);

    placeholder_contents.extend(article.metadata.widgets.render_to_placeholder_content());

    Ok(ArticlePage {
        article,
        html: placeholder
            .partially_fill_placeholders(&html_templates.article, |name| {
                placeholder_contents.get(name).cloned()
            })
            .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))?,
        path: Path::new(&article.group).join(format!("{}.html", &article.slug)),
    })
}
