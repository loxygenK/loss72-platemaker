use std::path::PathBuf;

use loss72_platemaker_core::fs::{Directory, FileSystemError};

#[derive(Debug, serde::Deserialize)]
pub struct ConfigurationScheme {
    pub html_template_dir: PathBuf,
    pub article_md_dir: PathBuf,
    pub destination: PathBuf,
}

#[derive(Debug)]
pub struct Configuration {
    pub html_template_dir: Directory,
    pub article_md_dir: Directory,
    pub destination: Directory,
}

impl TryFrom<ConfigurationScheme> for Configuration {
    type Error = FileSystemError;

    fn try_from(value: ConfigurationScheme) -> Result<Self, Self::Error> {
        Ok(Configuration {
            html_template_dir: Directory::new(value.html_template_dir)?,
            article_md_dir: Directory::new(value.article_md_dir)?,
            destination: Directory::new(value.destination)?,
        })
    }
}
