use std::path::Path;

use loss72_platemaker_construct::{copy_dir_recursively, copy_files, copy_individual_file};
use loss72_platemaker_core::{
    fs::{Directory, File},
    log,
};
use loss72_platemaker_markdown::{MarkdownProcessError, parse_markdown};
use loss72_platemaker_structure::{ArticleFile, ArticleGroup, AssetFile, ContentDirectory};
use loss72_platemaker_website::{
    WebsiteGenerationError, generate_article_html, generate_index_html, get_webpage_construction, load_templates,
};

use crate::{config::Configuration, error::report_error};

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error(transparent)]
    Markdown(#[from] MarkdownProcessError),

    #[error(transparent)]
    WebsiteGeneration(#[from] WebsiteGenerationError),

    #[error(transparent)]
    FileCopy(#[from] std::io::Error),
}

pub type TaskResult<T> = Result<T, TaskError>;

pub fn full_build(config: &Configuration) -> TaskResult<()> {
    log!(job_start: "Building all articles in {}", config.article_md_dir.path().display());

    let content_dir = ContentDirectory::new(&config.article_md_dir)?;

    log!(ok: "Discovered {} articles", content_dir.markdown_files.len());

    let result = Ok(())
        .and_then(|_| build_files(config, &content_dir.markdown_files, true))
        .and_then(|_| copy_template_files(config))
        .and_then(|_| copy_asset_files(config, &content_dir.article_group));

    if result.is_ok() {
        log!(job_end: "Successfully built all articles in {}", config.article_md_dir.path().display())
    }

    result
}

pub fn build_files(config: &Configuration, files: &[ArticleFile], full_build: bool) -> TaskResult<()> {
    let mut files = files.iter().peekable();

    if files.peek().is_none() {
        return Ok(());
    }

    log!(section: "Loading HTML from {}", config.html_template_dir.path().display());
    let html_templates = load_templates(&config.html_template_dir)?;

    let articles = files
        .filter_map(|file| parse_markdown(file).inspect_err(report_error).ok())
        .collect::<Vec<_>>();

    log!(ok: "Built {} articles", articles.len());
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
            return Err(error.into());
        }
    };

    let index_page = if full_build {
        Some(generate_index_html(&html_templates, htmls.as_slice())?)
    } else {
        None
    };

    log!(section: "Writing pages to the file system");

    let construction = get_webpage_construction(index_page.as_ref(), htmls.as_slice());
    let plan = construction.plan(config.destination.path());
    plan.execute()?;

    log!(ok: "Wrote pages");

    Ok(())
}

pub fn copy_template_files(config: &Configuration) -> TaskResult<()> {
    log!(section: "Copying files in template directory");

    copy_dir_recursively(
        &config.html_template_dir,
        &config.destination,
        &["_article.html".into()],
    )?;

    Ok(())
}

pub fn copy_asset_files(config: &Configuration, article_group: &[ArticleGroup]) -> TaskResult<()> {
    log!(section: "Copying asset files in article directory");

    let directories = article_group
        .iter()
        .flat_map(|group| {
            let dir = Directory::new(config.article_md_dir.path().join(group.group_dir_path()));
            let dir = match dir {
                Ok(dir) => dir,
                Err(e) => return Some(Err(e)),
            };

            dir.get_child("assets")
                .map(|dir| dir.map(|dir| (dir, group)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for (dir, group) in &directories {
        copy_dir_recursively(
            dir,
            &config.destination.get_or_mkdir_child(
                Path::new(".")
                    .join("articles")
                    .join(group.group_dir_flat_path())
                    .join("assets"),
            )?,
            &[],
        )?;
    }

    Ok(())
}

// TODO: Implement article asset copying feature to...
//         - full build. For each group, check if `assets` directory exists,
//           and copy files recursively if exists.
//         - watch. Just copy individual file using `copy_files`

pub fn copy_individual_template_files(config: &Configuration, files: &[File]) -> TaskResult<()> {
    if files.is_empty() {
        return Ok(());
    }

    log!(job_start: "Updating template files");

    let template_file = config.html_template_dir.path().join("_article.html");

    if files
        .iter()
        .any(|file| file.path() == template_file.as_path())
    {
        log!(warn: "Article page template file is updated! Rebuilding all articles.");
        full_build(config)?;
    }

    copy_files(&config.html_template_dir, &config.destination, files)?;

    log!(job_end: "Updated template files");

    Ok(())
}

pub fn copy_individual_assets_files(config: &Configuration, files: &[AssetFile]) -> TaskResult<()> {
    if files.is_empty() {
        return Ok(());
    }

    log!(job_start: "Updating asset files");

    for file in files {
        let file_root = config
            .article_md_dir
            .get_child(file.group.group_dir_path().join("assets"))
            .expect("assets directory to be exist")?;
        let dest_dir = &config.destination.get_or_mkdir_child(
            Path::new(".")
                .join("articles")
                .join(file.group.group_dir_flat_path())
                .join("assets"),
        )?;

        copy_individual_file(&file_root, dest_dir, file.file())?;
    }

    log!(job_end: "Updated asset files");

    Ok(())
}
