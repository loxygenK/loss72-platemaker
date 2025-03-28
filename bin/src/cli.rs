use std::path::{Path, PathBuf};

use clap::Parser;

use crate::cmd::watch::WatchParam;

#[derive(Parser, Debug)]
#[command()]
pub enum Commands {
    Build(BuildArgs),
    Watch(WatchArgs),
}

/// Build the website from a set of Markdown file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct BuildArgs {
    /// Path to config,
    #[arg(short, long, default_value = "./platemaker.toml")]
    pub config: PathBuf,
}

/// Watch for the file change, and update the website as necessary
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct WatchArgs {
    /// Path to config,
    #[arg(short, long, default_value = "./platemaker.toml")]
    pub config: PathBuf,

    /// Full build before watching. Failing to this build does not abort watching.
    #[arg(short, long, default_value_t = false)]
    pub build_first: bool,
}

impl Commands {
    pub fn config(&self) -> &Path {
        match self {
            Commands::Build(build_args) => &build_args.config,
            Commands::Watch(watch_args) => &watch_args.config,
        }
    }
}

impl From<WatchArgs> for WatchParam {
    fn from(value: WatchArgs) -> Self {
        Self {
            build_first: value.build_first,
        }
    }
}
