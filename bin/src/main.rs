#![deny(clippy::unwrap_used)]

pub mod build_tasks;
mod cli;
pub mod cmd;
mod config;
pub mod error;

use std::path::Path;

use anyhow::Context;
use clap::Parser;
use cli::Commands;
use cmd::{build::full_build, watch::watch_for_change};
use config::{Configuration, ConfigurationScheme};
use error::report_if_fail;
use loss72_platemaker_core::{fs::File, log};

fn main() -> Result<(), &'static str> {
    report_if_fail(|| {
        let args = Commands::parse();

        let config = read_config(args.config())?;

        println!();
        match args {
            Commands::Build(_) => build(&config),
            Commands::Watch(_) => watch(&config),
        }
    })
    .map_err(|_| "Failed due to the error above")
}

fn build(config: &Configuration) -> Result<(), anyhow::Error> {
    Ok(full_build(config)?)
}

fn watch(config: &Configuration) -> Result<(), anyhow::Error> {
    Ok(watch_for_change(config)?)
}

fn read_config(path: &Path) -> Result<Configuration, anyhow::Error> {
    log!(section: "Reading configuration {}", path.display());

    File::new(path)
        .context("Configuration file is not present or not available for reading")
        .and_then(|file| {
            file.read_to_string()
                .context("Failed to read configuration")
        })
        .and_then(|content| {
            toml::from_str::<ConfigurationScheme>(&content)
                .context("Configuration file is not valid")
        })
        .and_then(|parsed_file| {
            Configuration::try_from(parsed_file)
                .context("Configuration contains invalid configuration")
        })
}
