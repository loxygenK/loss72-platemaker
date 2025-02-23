use loss72_platemaker_construct::copy_dir_recursively;
use loss72_platemaker_core::{fs::File, log};
use loss72_platemaker_markdown::{MarkdownProcessError, parse_markdown};
use loss72_platemaker_website::{
    WebsiteGenerationError, generate_article_html, get_webpage_construction, load_templates,
};

use crate::config::Configuration;

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("There was an error during I/O operation: {0}")]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    Markdown(#[from] MarkdownProcessError),

    #[error(transparent)]
    WebsiteGeneration(#[from] WebsiteGenerationError),
}

pub type TaskResult<T> = Result<T, TaskError>;

pub fn full_build(config: &Configuration) -> TaskResult<()> {
    log!(job_start: "Building all articles in {}", config.article_md_dir.path().display());

    let result = build_files(
        config,
        config.article_md_dir.try_iter_tree()?.filter_map(|file| {
            if let Err(error) = &file {
                log!(warn: "There was an error during traversing direcotry: {}", error);
            }

            file.ok()
        }),
    );

    match result {
        Ok(()) => {
            log!(job_end: "Successfully built all articles in {}", config.article_md_dir.path().display())
        }
        Err(e) => panic!("{}", e),
    }

    result
}

pub fn build_files(config: &Configuration, files: impl Iterator<Item = File>) -> TaskResult<()> {
    log!(section: "Loading HTML from {}", config.html_template_dir.path().display());
    let html_templates = load_templates(&config.html_template_dir)?;

    let maybe_articles = files
        .filter(|file| file.path().extension().is_some_and(|ext| ext == "md"))
        .map(|file| parse_markdown(&config.article_md_dir, file))
        .collect::<Result<Vec<_>, _>>();

    let articles = match maybe_articles {
        Ok(articles) => {
            log!(ok: "Built all {} articles", articles.len());
            articles
        }
        Err(error) => {
            log!(warn: "There was an error during parse:\n{}", error);
            return Err(error.into());
        }
    };

    log!(section: "Generating HTML contents for articles");

    let htmls = articles
        .iter()
        .map(|article| generate_article_html(&html_templates, article))
        .collect::<Result<Vec<_>, _>>();

    let htmls = match htmls {
        Ok(htmls) => {
            log!(ok: "Generated all {} article pages", articles.len());
            htmls
        }
        Err(error) => {
            log!(warn: "There was an error during article page generation:\n{}", error);
            return Err(error.into());
        }
    };

    log!(section: "Writing pages to the file system");

    let construction = get_webpage_construction(htmls.as_slice());
    let plan = construction.plan(config.destination.path());
    plan.execute()?;

    log!(ok: "Wrote pages");

    log!(section: "Copying files in template directory");

    copy_dir_recursively(
        &config.html_template_dir,
        &config.destination,
        &["_article.html".into()],
    )?;

    Ok(())
}
