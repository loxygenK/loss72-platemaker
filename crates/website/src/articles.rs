use loss72_platemaker_construct::ConstructFile;
use loss72_platemaker_core::{log, model::{Article, GenerationContext}, util::get_slice_by_char};
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
    ctx: &GenerationContext,
) -> OutputResult<IndexPage> {
    log!(section: "Generating HTML for index page");

    if ctx.release {
        log!(step: "Using release build!");
    }

    let placeholder = Placeholder::from_strs("${", "}", None)
        .expect("Regex is validated to include the capture group");

    // We create list elements first
    let article_tag_iter = article
        .iter()
        .map(|page| {
            let mut placeholder_contents = article_to_placeholder_content(page.article, ctx);
                placeholder_contents.insert(
                "url",
                Path::new("/articles")
                    .join(&page.path)
                    .to_string_lossy()
                    .to_string(),
            );

            placeholder
                .partially_fill_placeholders(&html_templates.index_list, |name| {
                    placeholder_contents.get(name).cloned()
                })
                .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))
        })
        .collect::<Result<String, _>>()?;

    let placeholder_contents = HashMap::from([
        ("articles", article_tag_iter),
        ("style", html_templates.index_style.clone()),
        ("if-debug", if ctx.release { "<!-- (if-debug: false) ".to_string() } else { "".to_string() }),
        ("end-if-debug", if ctx.release { " (end-if-debug: false) -->".to_string() } else { "".to_string() }),
        ("if-release", if ctx.release { "".to_string() } else { "<!-- (if-release: false) ".to_string() }),
        ("end-if-release", if ctx.release { "".to_string() } else { " (end-if-release: false) -->".to_string() }),
    ]);

    Ok(IndexPage {
        path: PathBuf::from("index.html"),
        html: placeholder
            .partially_fill_placeholders(&html_templates.index, |name| {
                placeholder_contents.get(name).cloned()
            })
            .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))?,
    })
}

pub fn generate_article_html<'article>(
    html_templates: &WebPageHtmlTemplates,
    article: &'article Article,
    ctx: &GenerationContext,
) -> OutputResult<ArticlePage<'article>> {
    log!(step: "Generating HTML for slug '{}'", &article.id.slug);

    let path = Path::new(&article.id.group).join(format!("{}.html", &article.id.slug));

    let placeholder = Placeholder::from_strs("${", "}", None)
        .expect("Regex is validated to include the capture group");

    let mut placeholder_contents = article_to_placeholder_content(article, ctx);
    placeholder_contents.insert("content", article.content.clone());
    placeholder_contents.insert(
        "path", 
                Path::new("/articles")
                    .join(&path)
                    .to_string_lossy()
                    .to_string());
    placeholder_contents.extend(article.metadata.widgets.render_to_placeholder_content());

    Ok(ArticlePage {
        article,
        html: placeholder
            .partially_fill_placeholders(&html_templates.article, |name| {
                placeholder_contents.get(name).cloned()
            })
            .map_err(|invalids| WebsiteGenerationError::InvalidPlaceholder(invalids.clone()))?,
        path,
    })
}

fn article_to_placeholder_content(article: &Article, ctx: &GenerationContext) -> HashMap<&'static str, String> {
    let (year, month, day) = article.id.date;

    HashMap::from([
        (
            "type_class",
            article
                .metadata
                .widgets
                .article_type
                .class_name()
                .to_string(),
        ),
        (
            "type_name",
            article
                .metadata
                .widgets
                .article_type
                .description()
                .to_string(),
        ),
        ("title", article.metadata.title.clone()),
        ("brief", article.metadata.brief.clone()),
        ("year", year.to_string()),
        ("month", month.to_string()),
        ("day", day.to_string()),
        ("MM", format!("{:02}", month)),
        ("DD", format!("{:02}", day)),
        ("if-debug", if ctx.release { "".to_string() } else { "<!-- (debug) ".to_string() }),
        ("end-if-debug", if ctx.release { "".to_string() } else { " (debug) -->".to_string() }),
        ("if-release", if ctx.release { "<!-- (release) ".to_string() } else { "".to_string() }),
        ("end-if-release", if ctx.release { " (release) -->".to_string() } else { "".to_string() }),
    ])
}

